package expo.modules.opendict

import expo.modules.kotlin.modules.Module
import expo.modules.kotlin.modules.ModuleDefinition
import uniffi.opendict_mobile.Dictionary

class OpenDictModule : Module() {
    private val dictionaries = mutableMapOf<Int, Dictionary>()
    private var nextHandle = 1

    override fun definition() = ModuleDefinition {
        Name("OpenDict")

        Function("open") { dir: String ->
            val dict = Dictionary(dir)
            val handle = nextHandle++
            dictionaries[handle] = dict
            handle
        }

        Function("close") { handle: Int ->
            dictionaries.remove(handle)
        }

        Function("lookup") { handle: Int, word: String ->
            try {
                dictionaries[handle]?.lookup(word)?.map {
                    mapOf("entryType" to it.entryType, "data" to it.data)
                }
            } catch (_: Exception) {
                null
            }
        }

        Function("lookupSynonym") { handle: Int, word: String ->
            try {
                dictionaries[handle]?.lookupSynonym(word)?.map {
                    mapOf("entryType" to it.entryType, "data" to it.data)
                }
            } catch (_: Exception) {
                null
            }
        }

        Function("wordList") { handle: Int ->
            dictionaries[handle]?.wordList() ?: emptyList()
        }

        Function("wordCount") { handle: Int ->
            dictionaries[handle]?.wordCount()?.toInt() ?: 0
        }

        Function("searchPrefix") { handle: Int, prefix: String, limit: Int ->
            dictionaries[handle]?.searchPrefix(prefix, limit.toUInt()) ?: emptyList()
        }

        Function("getInfo") { handle: Int ->
            val info = dictionaries[handle]?.info()
            if (info != null) {
                mapOf(
                    "name" to info.name,
                    "author" to info.author,
                    "description" to info.description,
                    "wordCount" to info.wordCount,
                )
            } else {
                mapOf("name" to "", "author" to "", "description" to "", "wordCount" to 0)
            }
        }
    }
}
