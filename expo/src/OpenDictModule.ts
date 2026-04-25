import { requireNativeModule } from 'expo-modules-core'

export interface DictEntry {
  entryType: string
  data: string
}

export interface DictInfo {
  name: string
  author: string
  description: string
  wordCount: number
}

interface OpenDictNativeModule {
  open(dir: string): number
  close(handle: number): void
  lookup(handle: number, word: string): DictEntry[] | null
  lookupSynonym(handle: number, word: string): DictEntry[] | null
  wordList(handle: number): string[]
  wordCount(handle: number): number
  searchPrefix(handle: number, prefix: string, limit: number): string[]
  getInfo(handle: number): DictInfo
}

export default requireNativeModule<OpenDictNativeModule>('OpenDict')
