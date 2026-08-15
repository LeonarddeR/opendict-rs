//! Opening a StarDict dictionary from in-memory buffers must behave
//! exactly like opening the same files from disk.

use std::path::PathBuf;

use opendict::Dictionary;
use opendict::stardict::StarDictDictionary;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn load(names: &[&str]) -> Vec<(String, Vec<u8>)> {
    names
        .iter()
        .map(|n| (n.to_string(), std::fs::read(fixture(n)).unwrap()))
        .collect()
}

#[test]
fn open_from_memory_matches_open_from_disk() {
    let disk = StarDictDictionary::open(&fixture(""), "testdict").unwrap();
    let mem = StarDictDictionary::open_from_memory(load(&[
        "testdict.ifo",
        "testdict.idx",
        "testdict.dict",
        "testdict.syn",
    ]))
    .unwrap();

    assert_eq!(mem.word_list(), disk.word_list());
    assert_eq!(mem.info().name, disk.info().name);

    let mem_entries = mem.lookup("foo").unwrap().unwrap();
    let disk_entries = disk.lookup("foo").unwrap().unwrap();
    assert_eq!(mem_entries.len(), disk_entries.len());
    assert_eq!(mem_entries[0].data, disk_entries[0].data);
}

#[test]
fn open_from_memory_reads_dictzip_payload() {
    let mem = StarDictDictionary::open_from_memory(load(&[
        "testdict.ifo",
        "testdict.idx",
        "testdict.dict.dz",
    ]))
    .unwrap();
    let entries = mem.lookup("foo").unwrap().unwrap();
    assert_eq!(entries[0].data, b"bar");
}

#[test]
fn open_from_memory_resolves_synonyms() {
    let disk = StarDictDictionary::open(&fixture(""), "testdict").unwrap();
    let mem = StarDictDictionary::open_from_memory(load(&[
        "testdict.ifo",
        "testdict.idx",
        "testdict.dict",
        "testdict.syn",
    ]))
    .unwrap();
    let word = disk
        .word_list()
        .first()
        .map(|w| w.to_string())
        .unwrap();
    let mem_syn = mem.lookup_synonym(&word).unwrap();
    let disk_syn = disk.lookup_synonym(&word).unwrap();
    assert_eq!(mem_syn.is_some(), disk_syn.is_some());
}

#[test]
fn open_from_memory_without_ifo_is_invalid_format() {
    let result = StarDictDictionary::open_from_memory(load(&["testdict.idx", "testdict.dict"]));
    assert!(matches!(result, Err(opendict::Error::InvalidFormat(_))));
}
