use napi::bindgen_prelude::*;
use napi_derive::napi;

use std::path::PathBuf;

#[napi(object)]
pub struct DictEntry {
    pub entry_type: String,
    pub data: String,
}

#[napi(object)]
pub struct DictInfo {
    pub name: String,
    pub author: String,
    pub description: String,
    pub word_count: u32,
}

#[napi]
pub struct Dictionary {
    inner: Box<dyn opendict::Dictionary + Send + Sync>,
}

#[napi]
impl Dictionary {
    #[napi(constructor)]
    pub fn new(dir: String) -> Result<Self> {
        let inner = opendict::open(PathBuf::from(&dir))
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(Dictionary { inner })
    }

    #[napi]
    pub fn lookup(&self, word: String) -> Result<Option<Vec<DictEntry>>> {
        let entries = self.inner.lookup(&word)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(entries.map(|es| {
            es.into_iter()
                .map(|e| DictEntry {
                    entry_type: e.type_id.to_string(),
                    data: String::from_utf8_lossy(&e.data).into_owned(),
                })
                .collect()
        }))
    }

    #[napi]
    pub fn lookup_synonym(&self, word: String) -> Result<Option<Vec<DictEntry>>> {
        let entries = self.inner.lookup_synonym(&word)
            .map_err(|e| Error::from_reason(e.to_string()))?;
        Ok(entries.map(|es| {
            es.into_iter()
                .map(|e| DictEntry {
                    entry_type: e.type_id.to_string(),
                    data: String::from_utf8_lossy(&e.data).into_owned(),
                })
                .collect()
        }))
    }

    #[napi]
    pub fn word_list(&self) -> Vec<String> {
        self.inner.word_list().into_iter().map(|s| s.to_string()).collect()
    }

    #[napi]
    pub fn word_count(&self) -> u32 {
        self.inner.word_count() as u32
    }

    #[napi]
    pub fn search_prefix(&self, prefix: String, limit: u32) -> Vec<String> {
        self.inner.search_prefix(&prefix, limit as usize)
    }

    #[napi(getter)]
    pub fn info(&self) -> DictInfo {
        let i = self.inner.info();
        DictInfo {
            name: i.name.clone(),
            author: i.author.clone(),
            description: i.description.clone(),
            word_count: i.word_count as u32,
        }
    }
}
