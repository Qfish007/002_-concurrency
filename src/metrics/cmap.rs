use anyhow::Result;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct CmapMetrics {
    data: Arc<DashMap<String, i64>>,
}
impl CmapMetrics {
    pub fn new() -> Self {
        CmapMetrics {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self.data.entry(key.into()).or_insert(0);
        *data += 1;

        Ok(())
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .data
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect())
    }
}
