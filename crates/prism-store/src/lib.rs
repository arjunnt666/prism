//! Snapshot persistence.

use parking_lot::Mutex;
use prism_core::{PrismError, Result, Snapshot, SnapshotId};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub trait SnapshotStore: Send + Sync {
    fn put(&self, snap: &Snapshot) -> Result<()>;
    fn get(&self, id: SnapshotId) -> Result<Snapshot>;
    fn list_for_query(&self, query_text: &str) -> Result<Vec<Snapshot>>;
    fn list_all(&self) -> Result<Vec<Snapshot>>;
}

pub struct MemoryStore { inner: Mutex<HashMap<SnapshotId, Snapshot>> }

impl MemoryStore {
    pub fn new() -> Self { Self { inner: Mutex::new(HashMap::new()) } }
}
impl Default for MemoryStore { fn default() -> Self { Self::new() } }

impl SnapshotStore for MemoryStore {
    fn put(&self, snap: &Snapshot) -> Result<()> { self.inner.lock().insert(snap.id, snap.clone()); Ok(()) }
    fn get(&self, id: SnapshotId) -> Result<Snapshot> {
        self.inner.lock().get(&id).cloned().ok_or_else(|| PrismError::NotFound(id.to_string()))
    }
    fn list_for_query(&self, query_text: &str) -> Result<Vec<Snapshot>> {
        let mut v: Vec<_> = self.inner.lock().values().filter(|s| s.query.text == query_text).cloned().collect();
        v.sort_by_key(|s| s.captured_at); Ok(v)
    }
    fn list_all(&self) -> Result<Vec<Snapshot>> {
        let mut v: Vec<_> = self.inner.lock().values().cloned().collect();
        v.sort_by_key(|s| s.captured_at); Ok(v)
    }
}

pub struct JsonDirStore { root: PathBuf }

impl JsonDirStore {
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }
    fn path_for(&self, id: SnapshotId) -> PathBuf { self.root.join(format!("{}.json", id)) }
}

impl SnapshotStore for JsonDirStore {
    fn put(&self, snap: &Snapshot) -> Result<()> {
        fs::write(self.path_for(snap.id), serde_json::to_string_pretty(snap)?)?; Ok(())
    }
    fn get(&self, id: SnapshotId) -> Result<Snapshot> {
        Ok(serde_json::from_str(&fs::read_to_string(self.path_for(id))?)?)
    }
    fn list_for_query(&self, query_text: &str) -> Result<Vec<Snapshot>> {
        Ok(self.list_all()?.into_iter().filter(|s| s.query.text == query_text).collect())
    }
    fn list_all(&self) -> Result<Vec<Snapshot>> {
        let mut out = Vec::new();
        if !self.root.exists() { return Ok(out); }
        for entry in fs::read_dir(&self.root)? {
            let path = entry?.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
            if let Ok(s) = serde_json::from_str::<Snapshot>(&fs::read_to_string(path)?) { out.push(s); }
        }
        out.sort_by_key(|s| s.captured_at); Ok(out)
    }
}

pub fn load_snapshot_file(path: impl AsRef<Path>) -> Result<Snapshot> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

pub fn save_snapshot_file(path: impl AsRef<Path>, snap: &Snapshot) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() { fs::create_dir_all(parent)?; }
    fs::write(path, serde_json::to_string_pretty(snap)?)?; Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use prism_core::{Device, Query};
    #[test]
    fn memory_roundtrip() {
        let store = MemoryStore::new();
        let snap = Snapshot::new(Query { text: "test".into(), locale: None, device: Device::Desktop }, "sample", vec![]);
        store.put(&snap).unwrap();
        assert_eq!(store.get(snap.id).unwrap().query.text, "test");
    }
}
