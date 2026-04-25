import { Dictionary } from '@opendict-rs/node'
import { strict as assert } from 'node:assert'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const fixtureDir = path.join(__dirname, '..', '..', 'tests', 'fixtures')

// Open the test fixtures (auto-detects StarDict from .ifo file)
const dict = new Dictionary(fixtureDir)

// Info
console.log(`Dictionary: ${dict.info.name}`)
console.log(`Words: ${dict.wordCount()}`)
assert.equal(dict.info.name, 'A foo-bar dictionary')
assert.equal(dict.wordCount(), 4)

// Lookup
const fooEntries = dict.lookup('foo')
assert.ok(fooEntries)
assert.equal(fooEntries.length, 1)
assert.equal(fooEntries[0].data, 'bar')

const anotherEntries = dict.lookup('another')
assert.ok(anotherEntries)
assert.equal(anotherEntries[0].data, 'translat')

// Miss
assert.equal(dict.lookup('nonexistent'), null)

// Word list
const words = dict.wordList()
assert.deepEqual(words.sort(), ['another', 'foo', 'lorem', 'some word'])

// Prefix search
const prefixed = dict.searchPrefix('lo', 10)
assert.deepEqual(prefixed, ['lorem'])

// Synonym lookup
const synEntries = dict.lookupSynonym('abc')
assert.ok(synEntries)
assert.equal(synEntries.length, 1)

console.log('\nAll tests passed!')
