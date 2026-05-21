use std::path::PathBuf;
use std::sync::Arc;

uniffi::setup_scaffolding!();

#[derive(uniffi::Record)]
pub struct DictEntry {
    pub entry_type: String,
    pub data: String,
}

#[derive(uniffi::Record)]
pub struct DictInfo {
    pub name: String,
    pub author: String,
    pub description: String,
    pub word_count: u64,
}

#[derive(uniffi::Object)]
pub struct Dictionary {
    inner: Box<dyn opendict::Dictionary + Send + Sync>,
}

#[uniffi::export]
impl Dictionary {
    #[uniffi::constructor]
    fn new(dir: String) -> Result<Arc<Self>, DictError> {
        let inner = opendict::open(PathBuf::from(&dir))
            .map_err(|e| DictError::LoadError { message: e.to_string() })?;
        Ok(Arc::new(Dictionary { inner }))
    }

    fn lookup(&self, word: String) -> Result<Option<Vec<DictEntry>>, DictError> {
        let entries = self
            .inner
            .lookup(&word)
            .map_err(|e| DictError::LookupError { message: e.to_string() })?;
        Ok(entries.map(|es| {
            es.into_iter()
                .map(|e| DictEntry {
                    entry_type: e.type_id.to_string(),
                    data: String::from_utf8_lossy(&e.data).into_owned(),
                })
                .collect()
        }))
    }

    fn lookup_synonym(&self, word: String) -> Result<Option<Vec<DictEntry>>, DictError> {
        let entries = self
            .inner
            .lookup_synonym(&word)
            .map_err(|e| DictError::LookupError { message: e.to_string() })?;
        Ok(entries.map(|es| {
            es.into_iter()
                .map(|e| DictEntry {
                    entry_type: e.type_id.to_string(),
                    data: String::from_utf8_lossy(&e.data).into_owned(),
                })
                .collect()
        }))
    }

    fn lookup_by_index(&self, index: u32) -> Result<Option<Vec<DictEntry>>, DictError> {
        let entries = self
            .inner
            .lookup_by_index(index as usize)
            .map_err(|e| DictError::LookupError { message: e.to_string() })?;
        Ok(entries.map(|es| {
            es.into_iter()
                .map(|e| DictEntry {
                    entry_type: e.type_id.to_string(),
                    data: String::from_utf8_lossy(&e.data).into_owned(),
                })
                .collect()
        }))
    }

    fn word_list(&self) -> Vec<String> {
        self.inner.word_list().into_iter().map(|s| s.to_string()).collect()
    }

    fn word_count(&self) -> u32 {
        self.inner.word_count() as u32
    }

    fn search_prefix(&self, prefix: String, limit: u32) -> Vec<String> {
        self.inner.search_prefix(&prefix, limit as usize)
    }

    fn info(&self) -> DictInfo {
        let i = self.inner.info();
        DictInfo {
            name: i.name.clone(),
            author: i.author.clone(),
            description: i.description.clone(),
            word_count: i.word_count as u64,
        }
    }
}

#[derive(Debug, uniffi::Error)]
pub enum DictError {
    LoadError { message: String },
    LookupError { message: String },
}

impl std::fmt::Display for DictError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DictError::LoadError { message } => write!(f, "{}", message),
            DictError::LookupError { message } => write!(f, "{}", message),
        }
    }
}
