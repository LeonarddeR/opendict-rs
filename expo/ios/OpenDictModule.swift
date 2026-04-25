import ExpoModulesCore

public class OpenDictModule: Module {
    private var dictionaries: [Int: Dictionary] = [:]
    private var nextHandle = 1

    public func definition() -> ModuleDefinition {
        Name("OpenDict")

        Function("open") { (dir: String) -> Int in
            let dict = try Dictionary(dir: dir)
            let handle = self.nextHandle
            self.nextHandle += 1
            self.dictionaries[handle] = dict
            return handle
        }

        Function("close") { (handle: Int) in
            self.dictionaries.removeValue(forKey: handle)
        }

        Function("lookup") { (handle: Int, word: String) -> [[String: String]]? in
            guard let dict = self.dictionaries[handle] else { return nil }
            return try? dict.lookup(word: word)?.map {
                ["entryType": $0.entryType, "data": $0.data]
            }
        }

        Function("lookupSynonym") { (handle: Int, word: String) -> [[String: String]]? in
            guard let dict = self.dictionaries[handle] else { return nil }
            return try? dict.lookupSynonym(word: word)?.map {
                ["entryType": $0.entryType, "data": $0.data]
            }
        }

        Function("wordList") { (handle: Int) -> [String] in
            guard let dict = self.dictionaries[handle] else { return [] }
            return dict.wordList()
        }

        Function("wordCount") { (handle: Int) -> Int in
            guard let dict = self.dictionaries[handle] else { return 0 }
            return Int(dict.wordCount())
        }

        Function("searchPrefix") { (handle: Int, prefix: String, limit: Int) -> [String] in
            guard let dict = self.dictionaries[handle] else { return [] }
            return dict.searchPrefix(prefix: prefix, limit: UInt32(limit))
        }

        Function("getInfo") { (handle: Int) -> [String: Any] in
            guard let dict = self.dictionaries[handle] else {
                return ["name": "", "author": "", "description": "", "wordCount": 0]
            }
            let info = dict.info()
            return [
                "name": info.name,
                "author": info.author,
                "description": info.description,
                "wordCount": info.wordCount,
            ]
        }
    }
}
