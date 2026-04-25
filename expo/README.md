# @opendict-rs/expo

Expo native module that reads StarDict and MDict dictionaries via the
[opendict-rs](https://github.com/callum-gander/opendict-rs) Rust core.
Compiles to a static library on iOS (XCFramework) and a shared library
on Android, exposed through uniffi-generated bindings.

## Install

```bash
npx expo install @opendict-rs/expo
```

This package contains prebuilt binaries — no Rust toolchain required
to consume it. iOS supports `arm64` device + simulator; Android ships
all four ABIs (`arm64-v8a`, `armeabi-v7a`, `x86`, `x86_64`).

## Usage

```ts
import { Dictionary } from '@opendict-rs/expo'
import * as FileSystem from 'expo-file-system'

// Open a directory containing a dictionary (StarDict or MDict; auto-detected).
// Native code uses std::fs which doesn't accept file:// URIs — strip the prefix.
const dir = (FileSystem.documentDirectory ?? '') + 'my-dict/'
const dict = new Dictionary(dir.replace(/^file:\/\//, ''))

console.log(dict.info.name, dict.wordCount())

const entries = dict.lookup('hello')
if (entries) {
  for (const e of entries) console.log(e.data)
}

const matches = dict.searchPrefix('he', 10)
console.log(matches)

dict.close()
```

## API

### `new Dictionary(dir: string)`

Opens the dictionary at `dir`. Format is detected automatically — StarDict
first (looks for `.ifo`), then MDict (looks for `.mdx`). Throws
`DictError.LoadError` if neither is found.

### `dict.info: DictInfo`

`{ name, author, description, wordCount }`.

### `dict.lookup(word: string): DictEntry[] | null`

Exact lookup. Returns `null` if the word isn't in the dictionary.
Each `DictEntry` has `{ entryType, data }`.

### `dict.lookupSynonym(word: string): DictEntry[] | null`

Lookup via the synonym/variant index (StarDict `.syn`). Returns `null`
if no match.

### `dict.searchPrefix(prefix: string, limit: number): string[]`

Up to `limit` words that start with `prefix`. Used for typeahead.

### `dict.wordList(): string[]`

Every headword. Can be huge — don't render it.

### `dict.wordCount(): number`

Same as `info.wordCount`, kept for cheap polling.

### `dict.close(): void`

Releases the underlying file mapping. Always call this in your component
cleanup.

## File system layout

The native code expects a *directory* — not a single file. Drop all the
files for a dictionary into one folder:

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

Open the folder, not any individual file.

## License

MIT — see [LICENSE](../LICENSE).
