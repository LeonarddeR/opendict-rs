# @opendict-rs/node

Node.js bindings for [opendict-rs](https://github.com/callum-gander/opendict-rs).
Reads StarDict and MDict dictionaries from JavaScript via a napi-rs
addon. Format is auto-detected.

Prebuilt binaries ship for macOS (x64 + arm64), Linux (x64 + arm64,
gnu + musl) and Windows (x64 + arm64) — npm pulls only the one for
your machine.

## Install

```bash
npm install @opendict-rs/node
```

No Rust toolchain required.

## Usage

```js
import { Dictionary } from '@opendict-rs/node'

const dict = new Dictionary('/path/to/dict-folder')

console.log(dict.info.name, dict.wordCount())

const entries = dict.lookup('hello')
if (entries) {
  for (const e of entries) console.log(e.data)
}

const matches = dict.searchPrefix('he', 10)
console.log(matches)
```

## API

### `new Dictionary(dir: string)`

Opens the dictionary at `dir`. The directory must contain either a
StarDict set (`.ifo` + `.idx` + `.dict`) or an MDict set (`.mdx`).
Throws if neither is found.

### `dict.info: DictInfo`

`{ name, author, description, wordCount }`.

### `dict.lookup(word: string): DictEntry[] | null`

Exact lookup. Returns `null` if the word isn't in the dictionary.

### `dict.lookupSynonym(word: string): DictEntry[] | null`

Lookup via the StarDict synonym index. `null` if no match.

### `dict.searchPrefix(prefix: string, limit: number): string[]`

Up to `limit` words that begin with `prefix`.

### `dict.wordList(): string[]`

Every headword. Big.

### `dict.wordCount(): number`

Headword count. Same as `info.wordCount`.

## File system layout

The directory passed to `new Dictionary(...)` should contain all the
files for one dictionary side-by-side:

```
StarDict/
  whatever.ifo
  whatever.idx
  whatever.dict        (or .dict.gz / .dict.dz)
  whatever.syn         (optional)

MDict/
  whatever.mdx
  whatever.mdd         (optional)
```

## License

MIT — see [LICENSE](../LICENSE).
