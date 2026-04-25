import OpenDictModule, { type DictEntry, type DictInfo } from './OpenDictModule'

export type { DictEntry, DictInfo }

export class Dictionary {
  private handle: number

  constructor(dir: string) {
    this.handle = OpenDictModule.open(dir)
  }

  close(): void {
    OpenDictModule.close(this.handle)
  }

  lookup(word: string): DictEntry[] | null {
    return OpenDictModule.lookup(this.handle, word)
  }

  lookupSynonym(word: string): DictEntry[] | null {
    return OpenDictModule.lookupSynonym(this.handle, word)
  }

  wordList(): string[] {
    return OpenDictModule.wordList(this.handle)
  }

  wordCount(): number {
    return OpenDictModule.wordCount(this.handle)
  }

  searchPrefix(prefix: string, limit: number = 10): string[] {
    return OpenDictModule.searchPrefix(this.handle, prefix, limit)
  }

  get info(): DictInfo {
    return OpenDictModule.getInfo(this.handle)
  }
}
