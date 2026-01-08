use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    data: Arc<RwLock<HashMap<String, i64>>>,
}
impl Metrics {
    pub fn new() -> Self {
        Metrics {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn inc(&self, key: impl Into<String>) -> Result<()> {
        let mut data = self
            .data
            .write()
            .map_err(|e| anyhow!("{:?}", e.to_string()))?;
        let counter = data.entry(key.into()).or_insert(0);
        *counter += 1;

        Ok(())
    }

    pub fn snapshot(&self) -> Result<HashMap<String, i64>> {
        Ok(self
            .data
            .read()
            .map_err(|e| anyhow!("{:?}", e.to_string()))?
            .clone())
    }
}
