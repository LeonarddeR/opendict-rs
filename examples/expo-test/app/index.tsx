import { useEffect, useState } from 'react'
import { ScrollView, StyleSheet, Text, View } from 'react-native'
import * as FileSystem from 'expo-file-system'
import { Dictionary } from '@opendict-rs/expo'

type Check = { name: string; pass: boolean; detail?: string }
type Sample = { label: string; value: string }

type DictReport = {
  dirName: string
  fatal?: string
  info?: { name: string; wordCount: number }
  checks: Check[]
  samples: Sample[]
}

export default function App() {
  const [reports, setReports] = useState<DictReport[]>([])
  const [status, setStatus] = useState('Discovering dicts...')

  useEffect(() => { run() }, [])

  async function run() {
    try {
      const dictsDir = (FileSystem.documentDirectory ?? '') + 'dicts/'
      const dictsDirInfo = await FileSystem.getInfoAsync(dictsDir)
      if (!dictsDirInfo.exists) {
        setStatus(
          'No dicts found. Push some via:\n' +
          '  ./scripts/setup-dicts.sh && ./scripts/push-dicts.sh',
        )
        return
      }

      const entries = await FileSystem.readDirectoryAsync(dictsDir)
      const dirs: string[] = []
      for (const name of entries) {
        const info = await FileSystem.getInfoAsync(dictsDir + name)
        if (info.exists && info.isDirectory) dirs.push(name)
      }
      dirs.sort()

      if (dirs.length === 0) {
        setStatus('dicts/ exists but is empty.')
        return
      }

      setStatus(`Running ${dirs.length} dict(s)...`)
      const out: DictReport[] = []
      for (const name of dirs) {
        out.push(await testDictionary(dictsDir + name + '/', name))
      }
      setReports(out)
      setStatus('')
    } catch (e: any) {
      setStatus(`Fatal: ${e?.message ?? String(e)}`)
    }
  }

  return (
    <View style={styles.container}>
      <Text style={styles.title}>opendict-rs · expo bindings</Text>
      {status !== '' && <Text style={styles.status}>{status}</Text>}
      <ScrollView style={{ flex: 1 }}>
        {reports.map(r => <ReportView key={r.dirName} report={r} />)}
      </ScrollView>
    </View>
  )
}

function ReportView({ report }: { report: DictReport }) {
  const total = report.checks.length
  const passed = report.checks.filter(c => c.pass).length
  const allPassed = total > 0 && passed === total

  return (
    <View style={styles.report}>
      <View style={styles.reportHeader}>
        <Text style={styles.reportTitle} numberOfLines={1}>{report.dirName}</Text>
        {report.fatal ? (
          <Text style={[styles.reportSummary, { color: '#b21d1d' }]}>FAIL</Text>
        ) : (
          <Text style={[styles.reportSummary, { color: allPassed ? '#0a7d2c' : '#b21d1d' }]}>
            {passed}/{total}
          </Text>
        )}
      </View>

      {report.info && (
        <Text style={styles.reportMeta}>
          {report.info.name || '(unnamed)'} · {report.info.wordCount.toLocaleString()} words
        </Text>
      )}

      {report.fatal && <Text style={styles.fatal}>{report.fatal}</Text>}

      {report.samples.length > 0 && (
        <View style={styles.samples}>
          {report.samples.map((s, i) => (
            <View key={i} style={styles.sample}>
              <Text style={styles.sampleLabel}>{s.label}</Text>
              <Text style={styles.sampleValue}>{s.value}</Text>
            </View>
          ))}
        </View>
      )}

      {report.checks.some(c => !c.pass) && (
        <View style={styles.failures}>
          {report.checks.filter(c => !c.pass).map((c, i) => (
            <Text key={i} style={styles.failureLine}>✗ {c.name}: {c.detail}</Text>
          ))}
        </View>
      )}
    </View>
  )
}

async function testDictionary(dir: string, dirName: string): Promise<DictReport> {
  const nativeDir = dir.replace(/^file:\/\//, '')
  const checks: Check[] = []
  const samples: Sample[] = []

  let dict: Dictionary
  try {
    dict = new Dictionary(nativeDir)
  } catch (e: any) {
    return {
      dirName,
      fatal: e?.message ?? String(e),
      checks: [],
      samples: [],
    }
  }

  let info: { name: string; wordCount: number } | undefined
  try {
    const i = dict.info
    info = { name: i.name, wordCount: i.wordCount }
    checks.push({ name: 'info readable', pass: true })
  } catch (e: any) {
    checks.push({ name: 'info readable', pass: false, detail: e?.message })
  }

  let words: string[] = []
  try {
    words = dict.wordList()
    checks.push({
      name: 'wordList returns array',
      pass: Array.isArray(words) && words.length > 0,
      detail: words.length === 0 ? 'empty' : undefined,
    })
  } catch (e: any) {
    checks.push({ name: 'wordList returns array', pass: false, detail: e?.message })
  }

  let wordCount = 0
  try {
    wordCount = dict.wordCount()
    checks.push({
      name: 'wordCount matches wordList length',
      pass: wordCount === words.length,
      detail: wordCount !== words.length ? `wordCount=${wordCount}, list=${words.length}` : undefined,
    })
  } catch (e: any) {
    checks.push({ name: 'wordCount', pass: false, detail: e?.message })
  }

  // Sample: a few words from the start of the list
  if (words.length > 0) {
    const sampled = words.slice(0, 5).map(w => w.length > 30 ? w.slice(0, 30) + '…' : w)
    samples.push({ label: 'sample words', value: sampled.join(', ') })
  }

  // Sample: actual lookup of the first word
  if (words.length > 0) {
    const word = words[0]
    try {
      const entries = dict.lookup(word)
      checks.push({
        name: 'lookup(first word) returns entries',
        pass: !!entries && entries.length > 0,
        detail: !entries ? 'null' : undefined,
      })
      if (entries && entries.length > 0) {
        const data = entries[0].data ?? ''
        const preview = data.length > 200 ? data.slice(0, 200) + '…' : data
        samples.push({ label: `lookup("${word}")`, value: preview || '(empty)' })
      }
    } catch (e: any) {
      checks.push({ name: 'lookup(first word)', pass: false, detail: e?.message })
    }
  }

  // Lookup of a guaranteed miss should return null
  try {
    const r = dict.lookup('__definitely_not_a_word_xyz__')
    checks.push({
      name: 'lookup(missing) returns null',
      pass: r === null,
      detail: r !== null ? `got ${r?.length} entries` : undefined,
    })
  } catch (e: any) {
    checks.push({ name: 'lookup(missing)', pass: false, detail: e?.message })
  }

  // Sample: prefix search
  if (words.length > 0) {
    const seed = words[0]
    const prefix = seed.slice(0, Math.min(2, seed.length))
    try {
      const matches = dict.searchPrefix(prefix, 5)
      checks.push({
        name: 'searchPrefix returns matches',
        pass: Array.isArray(matches) && matches.length > 0 && matches.length <= 5,
        detail: matches?.length === 0 ? 'no matches' : undefined,
      })
      const display = matches.map(m => m.length > 25 ? m.slice(0, 25) + '…' : m)
      samples.push({ label: `searchPrefix("${prefix}", 5)`, value: display.join(', ') || '(empty)' })
    } catch (e: any) {
      checks.push({ name: 'searchPrefix', pass: false, detail: e?.message })
    }
  }

  // Sample: synonym lookup of the first word
  if (words.length > 0) {
    const word = words[0]
    try {
      const syn = dict.lookupSynonym(word)
      checks.push({ name: 'lookupSynonym does not throw', pass: true })
      if (syn && syn.length > 0) {
        const data = syn[0].data ?? ''
        const preview = data.length > 120 ? data.slice(0, 120) + '…' : data
        samples.push({ label: `lookupSynonym("${word}")`, value: preview || '(empty)' })
      } else {
        samples.push({ label: `lookupSynonym("${word}")`, value: '(none)' })
      }
    } catch (e: any) {
      checks.push({ name: 'lookupSynonym', pass: false, detail: e?.message })
    }
  }

  try { dict.close() } catch {}

  return { dirName, info, checks, samples }
}

const styles = StyleSheet.create({
  container: { flex: 1, padding: 16, paddingTop: 60, backgroundColor: '#fafafa' },
  title: { fontSize: 20, fontWeight: '700', marginBottom: 8 },
  status: { color: '#666', marginBottom: 8 },
  report: {
    backgroundColor: '#fff',
    padding: 12,
    borderRadius: 8,
    marginBottom: 10,
    borderWidth: 1,
    borderColor: '#eee',
  },
  reportHeader: { flexDirection: 'row', justifyContent: 'space-between', alignItems: 'center' },
  reportTitle: { fontSize: 14, fontWeight: '600', flex: 1, marginRight: 8 },
  reportSummary: { fontSize: 13, fontWeight: '700' },
  reportMeta: { fontSize: 11, color: '#666', marginTop: 2, marginBottom: 8 },
  fatal: { fontSize: 11, color: '#b21d1d', fontFamily: 'Menlo' },
  samples: { marginTop: 4 },
  sample: { marginBottom: 6 },
  sampleLabel: { fontSize: 10, color: '#888', textTransform: 'uppercase', letterSpacing: 0.5 },
  sampleValue: { fontSize: 12, color: '#222', fontFamily: 'Menlo', marginTop: 1 },
  failures: { marginTop: 6, paddingTop: 6, borderTopWidth: 1, borderTopColor: '#f0f0f0' },
  failureLine: { fontSize: 11, color: '#b21d1d' },
})
