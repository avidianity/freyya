use std::collections::HashMap;
use std::hash::{DefaultHasher, Hasher};
use std::sync::Arc;

use crate::{Rakaly, Refusal, Snapshot, read_save};

#[derive(Debug, Clone, PartialEq)]
pub enum SaveRead {
    New(Arc<Snapshot>),
    AlreadyRead(Arc<Snapshot>),
}

#[derive(Debug, Default)]
pub struct SaveReader {
    snapshots: HashMap<SaveContent, Arc<Snapshot>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SaveContent {
    length: usize,
    digest: u64,
}

impl SaveContent {
    fn of(save: &[u8]) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write(save);
        Self {
            length: save.len(),
            digest: hasher.finish(),
        }
    }
}

impl SaveReader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, save: &[u8], rakaly: &Rakaly) -> Result<SaveRead, Refusal> {
        let content = SaveContent::of(save);
        if let Some(snapshot) = self.snapshots.get(&content) {
            return Ok(SaveRead::AlreadyRead(Arc::clone(snapshot)));
        }
        let snapshot = Arc::new(read_save(save, rakaly)?);
        self.snapshots.insert(content, Arc::clone(&snapshot));
        Ok(SaveRead::New(snapshot))
    }
}
