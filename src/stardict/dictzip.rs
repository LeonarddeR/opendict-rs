//! dictzip random-access reader.
//!
//! A `.dict.dz` file is a gzip-compatible container: a standard gzip stream
//! whose `FEXTRA` field carries an `"RA"` (random access) subfield describing a
//! chunk table. The uncompressed payload is split into fixed-size chunks of
//! `CHLEN` bytes, and the deflate state is flushed at every chunk boundary so
//! each chunk can be inflated independently. The RA subfield records the
//! *compressed* length of every chunk, letting a reader seek to and inflate only
//! the chunk(s) covering a wanted byte range instead of decompressing the whole
//! file.
//!
//! RA subfield `data` layout — all little-endian `u16`:
//!
//! ```text
//! VER          (= 1)
//! CHLEN        uncompressed bytes per chunk
//! CHCNT        number of chunks
//! CHCNT × u16  compressed length of each chunk
//! ```
//!
//! so `LEN == 6 + 2 * CHCNT`.

use std::io::Read;
use std::path;

use flate2::read::DeflateDecoder;
use memmap2::Mmap;

use crate::error::Error;

// gzip FLG bits (RFC 1952).
const FHCRC: u8 = 1 << 1;
const FEXTRA: u8 = 1 << 2;
const FNAME: u8 = 1 << 3;
const FCOMMENT: u8 = 1 << 4;

/// Backing bytes of a `.dz` stream: a file mapping or an owned buffer.
#[derive(Debug)]
enum DzSource {
    Mapped(Mmap),
    Owned(Vec<u8>),
}

impl DzSource {
    fn bytes(&self) -> &[u8] {
        match self {
            DzSource::Mapped(m) => m,
            DzSource::Owned(v) => v,
        }
    }
}

/// A `.dict.dz` stream with a parsed dictzip chunk table.
#[derive(Debug)]
pub struct DictzipReader {
    source: DzSource,
    chunk_len: usize,
    /// File offset of each chunk's compressed data, with a trailing sentinel at
    /// the end of the last chunk. Length is `CHCNT + 1`.
    comp_offsets: Vec<usize>,
}

impl DictzipReader {
    /// Open a `.dz` file and parse its dictzip RA header.
    ///
    /// Returns `Ok(None)` if the file is a plain gzip stream without a dictzip
    /// `"RA"` subfield, so the caller can fall back to whole-file decompression.
    pub fn open(file: &path::Path) -> crate::Result<Option<DictzipReader>> {
        let f = std::fs::File::open(file)?;
        // SAFETY: dictionary files are read-only; we never mutate the mapping.
        let mmap = unsafe { Mmap::map(&f)? };
        Ok(parse_header(&mmap)?.map(|(chunk_len, comp_offsets)| DictzipReader {
            source: DzSource::Mapped(mmap),
            chunk_len,
            comp_offsets,
        }))
    }

    /// Build a reader over an in-memory `.dz` stream.
    ///
    /// Returns the buffer back as the inner `Err` when it is plain gzip
    /// without an RA subfield, so the caller can fall back to whole-stream
    /// decompression without copying.
    pub(crate) fn from_owned_bytes(
        data: Vec<u8>,
    ) -> crate::Result<Result<DictzipReader, Vec<u8>>> {
        match parse_header(&data)? {
            Some((chunk_len, comp_offsets)) => Ok(Ok(DictzipReader {
                source: DzSource::Owned(data),
                chunk_len,
                comp_offsets,
            })),
            None => Ok(Err(data)),
        }
    }

    /// Inflate and return `size` bytes starting at uncompressed `offset`.
    pub fn read(&self, offset: u64, size: u32) -> crate::Result<Vec<u8>> {
        read_range(self.source.bytes(), self.chunk_len, &self.comp_offsets, offset, size)
    }
}

/// Parse the gzip + dictzip RA header from `d`.
///
/// Returns `Ok(Some((chunk_len, comp_offsets)))` for a dictzip file,
/// `Ok(None)` for a plain gzip stream (no RA subfield), or an error for a
/// malformed RA table.
fn parse_header(d: &[u8]) -> crate::Result<Option<(usize, Vec<usize>)>> {
    // Fixed 10-byte gzip header + 2-byte XLEN minimum.
    if d.len() < 12 || d[0] != 0x1f || d[1] != 0x8b || d[2] != 0x08 {
        return Ok(None); // not a gzip/deflate stream
    }
    let flg = d[3];
    if flg & FEXTRA == 0 {
        return Ok(None); // no extra field => not dictzip
    }

    let xlen = u16::from_le_bytes([d[10], d[11]]) as usize;
    let extra_start: usize = 12;
    let extra_end = extra_start
        .checked_add(xlen)
        .filter(|&e| e <= d.len())
        .ok_or_else(|| Error::InvalidFormat("dictzip: FEXTRA overruns file".into()))?;
    let extra = &d[extra_start..extra_end];

    let ra = match find_ra_subfield(extra)? {
        Some(r) => r,
        None => return Ok(None), // gzip with extra but no RA => not dictzip
    };

    if ra.len() < 6 {
        return Err(Error::InvalidFormat(
            "dictzip: RA subfield too short".into(),
        ));
    }
    let ver = u16::from_le_bytes([ra[0], ra[1]]);
    if ver != 1 {
        return Err(Error::InvalidFormat(format!(
            "dictzip: unsupported RA version {ver}"
        )));
    }
    let chunk_len = u16::from_le_bytes([ra[2], ra[3]]) as usize;
    let chunk_count = u16::from_le_bytes([ra[4], ra[5]]) as usize;
    if chunk_len == 0 {
        return Err(Error::InvalidFormat("dictzip: zero CHLEN".into()));
    }
    let expected_len = 6 + 2 * chunk_count;
    if ra.len() != expected_len {
        return Err(Error::InvalidFormat(format!(
            "dictzip: RA length {} != {} (6 + 2*CHCNT)",
            ra.len(),
            expected_len
        )));
    }

    // Compressed data starts after the full header: skip FNAME / FCOMMENT /
    // FHCRC according to the FLG bits.
    let mut pos = extra_end;
    if flg & FNAME != 0 {
        pos = skip_cstr(d, pos)?;
    }
    if flg & FCOMMENT != 0 {
        pos = skip_cstr(d, pos)?;
    }
    if flg & FHCRC != 0 {
        pos = pos
            .checked_add(2)
            .filter(|&p| p <= d.len())
            .ok_or_else(|| Error::InvalidFormat("dictzip: truncated FHCRC".into()))?;
    }

    // Prefix-sum the per-chunk compressed lengths into absolute file offsets.
    let mut comp_offsets = Vec::with_capacity(chunk_count + 1);
    comp_offsets.push(pos);
    let mut off = pos;
    for i in 0..chunk_count {
        let clen = u16::from_le_bytes([ra[6 + 2 * i], ra[6 + 2 * i + 1]]) as usize;
        off = off
            .checked_add(clen)
            .ok_or_else(|| Error::InvalidFormat("dictzip: chunk offset overflow".into()))?;
        comp_offsets.push(off);
    }
    // Last chunk's compressed data must fit before the 8-byte gzip footer.
    if off + 8 > d.len() {
        return Err(Error::InvalidFormat(
            "dictzip: chunk table overruns file".into(),
        ));
    }

    Ok(Some((chunk_len, comp_offsets)))
}

/// Inflate `size` bytes at uncompressed `offset` from the chunked stream `d`.
fn read_range(
    d: &[u8],
    chunk_len: usize,
    comp_offsets: &[usize],
    offset: u64,
    size: u32,
) -> crate::Result<Vec<u8>> {
    let size = size as usize;
    if size == 0 {
        return Ok(Vec::new());
    }
    let offset = offset as usize;
    let end = offset
        .checked_add(size)
        .ok_or_else(|| Error::InvalidFormat("dictzip: entry offset+size overflow".into()))?;

    let chunk_count = comp_offsets.len() - 1;
    let first = offset / chunk_len;
    let last = (end - 1) / chunk_len;
    if last >= chunk_count {
        return Err(Error::InvalidFormat(format!(
            "dictzip: entry offset {offset}+{size} exceeds {chunk_count} chunks"
        )));
    }

    let mut buf = Vec::with_capacity((last - first + 1) * chunk_len);
    for i in first..=last {
        let comp = &d[comp_offsets[i]..comp_offsets[i + 1]];
        let mut decoder = DeflateDecoder::new(comp);
        if i + 1 == chunk_count {
            // Final chunk: the deflate stream is BFINAL-terminated, so read to
            // the end (it may be shorter than CHLEN).
            decoder.read_to_end(&mut buf)?;
        } else {
            // Interior chunk: the stream is only flush-terminated (no BFINAL),
            // so pull exactly CHLEN bytes rather than reading to stream end.
            let base = buf.len();
            buf.resize(base + chunk_len, 0);
            decoder.read_exact(&mut buf[base..])?;
        }
    }

    let local = offset - first * chunk_len;
    if local + size > buf.len() {
        return Err(Error::InvalidFormat(
            "dictzip: decompressed data shorter than the requested range".into(),
        ));
    }
    Ok(buf[local..local + size].to_vec())
}

/// Scan a gzip FEXTRA area for the dictzip `"RA"` subfield, returning its data.
fn find_ra_subfield(extra: &[u8]) -> crate::Result<Option<&[u8]>> {
    let mut p = 0;
    while p + 4 <= extra.len() {
        let si1 = extra[p];
        let si2 = extra[p + 1];
        let len = u16::from_le_bytes([extra[p + 2], extra[p + 3]]) as usize;
        let data_start = p + 4;
        let data_end = data_start
            .checked_add(len)
            .filter(|&e| e <= extra.len())
            .ok_or_else(|| Error::InvalidFormat("dictzip: subfield overruns FEXTRA".into()))?;
        if si1 == b'R' && si2 == b'A' {
            return Ok(Some(&extra[data_start..data_end]));
        }
        p = data_end;
    }
    Ok(None)
}

/// Return the offset just past the NUL terminator of a C string at `start`.
fn skip_cstr(d: &[u8], start: usize) -> crate::Result<usize> {
    let rel = d[start..]
        .iter()
        .position(|&b| b == 0)
        .ok_or_else(|| Error::InvalidFormat("dictzip: unterminated header string".into()))?;
    Ok(start + rel + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    /// Raw-deflate a buffer into a complete (BFINAL-terminated) stream.
    fn deflate_raw(data: &[u8]) -> Vec<u8> {
        use flate2::{Compression, write::DeflateEncoder};
        use std::io::Write;
        let mut e = DeflateEncoder::new(Vec::new(), Compression::default());
        e.write_all(data).unwrap();
        e.finish().unwrap()
    }

    /// Build a valid dictzip container over `data` with the given chunk length.
    fn build_dictzip(data: &[u8], chunk_len: usize) -> Vec<u8> {
        let chunks: Vec<&[u8]> = if data.is_empty() {
            vec![&[][..]]
        } else {
            data.chunks(chunk_len).collect()
        };
        let comp: Vec<Vec<u8>> = chunks.iter().map(|c| deflate_raw(c)).collect();

        let mut ra = Vec::new();
        ra.extend_from_slice(&1u16.to_le_bytes()); // VER
        ra.extend_from_slice(&(chunk_len as u16).to_le_bytes()); // CHLEN
        ra.extend_from_slice(&(comp.len() as u16).to_le_bytes()); // CHCNT
        for c in &comp {
            ra.extend_from_slice(&(c.len() as u16).to_le_bytes());
        }

        let mut extra = Vec::new();
        extra.extend_from_slice(b"RA");
        extra.extend_from_slice(&(ra.len() as u16).to_le_bytes());
        extra.extend_from_slice(&ra);

        let mut out = Vec::new();
        out.extend_from_slice(&[0x1f, 0x8b, 0x08]); // magic + CM=deflate
        out.push(FEXTRA); // FLG
        out.extend_from_slice(&[0, 0, 0, 0]); // MTIME
        out.push(0); // XFL
        out.push(0xff); // OS = unknown
        out.extend_from_slice(&(extra.len() as u16).to_le_bytes()); // XLEN
        out.extend_from_slice(&extra);
        for c in &comp {
            out.extend_from_slice(c);
        }
        let mut crc = flate2::Crc::new();
        crc.update(data);
        out.extend_from_slice(&crc.sum().to_le_bytes()); // CRC32
        out.extend_from_slice(&(data.len() as u32).to_le_bytes()); // ISIZE
        out
    }

    #[test]
    fn plain_gzip_is_not_dictzip() {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(b"hello world").unwrap();
        let gz = e.finish().unwrap();
        assert!(parse_header(&gz).unwrap().is_none());
    }

    #[test]
    fn multi_chunk_roundtrip() {
        let data: Vec<u8> = (0..200u32).map(|i| (i % 251) as u8).collect();
        let chunk_len = 16;
        let dz = build_dictzip(&data, chunk_len);
        let (clen, offsets) = parse_header(&dz).unwrap().unwrap();
        assert_eq!(clen, chunk_len);
        assert_eq!(offsets.len(), data.len().div_ceil(chunk_len) + 1);

        // Reads within one chunk, spanning two, spanning three, and at the tail.
        for &(off, size) in &[
            (0u64, 4u32), // start of chunk 0
            (10, 4),      // within chunk 0
            (14, 6),      // spans chunk 0/1 boundary
            (30, 40),     // spans three chunks
            (190, 10),    // ends exactly at the tail (len 200)
        ] {
            let got = read_range(&dz, clen, &offsets, off, size).unwrap();
            let want = &data[off as usize..off as usize + size as usize];
            assert_eq!(got, want, "off={off} size={size}");
        }
    }

    #[test]
    fn read_past_end_errors() {
        let data: Vec<u8> = (0..40u8).collect();
        let dz = build_dictzip(&data, 16);
        let (clen, offsets) = parse_header(&dz).unwrap().unwrap();
        assert!(read_range(&dz, clen, &offsets, 38, 10).is_err());
    }

    #[test]
    fn bad_version_rejected() {
        let data: Vec<u8> = (0..40u8).collect();
        let mut dz = build_dictzip(&data, 16);
        // VER is the first u16 of the RA data: 12-byte header + "RA"(2) + LEN(2).
        dz[16] = 2;
        assert!(parse_header(&dz).is_err());
    }

    #[test]
    fn truncated_ra_table_rejected() {
        let data: Vec<u8> = (0..40u8).collect();
        let mut dz = build_dictzip(&data, 16);
        // Corrupt CHCNT (3rd u16 of RA data, at header 12 + 2 + 2 + 4 = 20) so
        // that LEN != 6 + 2*CHCNT.
        dz[20] = dz[20].wrapping_add(1);
        assert!(parse_header(&dz).is_err());
    }

    #[test]
    fn single_chunk_fixture_matches_plain_dict() {
        // testdict.dict.dz decodes to exactly testdict.dict.
        let dz_bytes = std::fs::read(fixture("testdict.dict.dz")).unwrap();
        let plain = std::fs::read(fixture("testdict.dict")).unwrap();
        let (clen, offsets) = parse_header(&dz_bytes).unwrap().unwrap();
        let got = read_range(&dz_bytes, clen, &offsets, 0, plain.len() as u32).unwrap();
        assert_eq!(got, plain);
    }
}
