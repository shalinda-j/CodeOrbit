//! Context management for the CodeOrbit extension.
//!
//! This module provides a way to share state and context between different
//! components of the CodeOrbit extension.

use crate::core::error::{Error, Result};
use chrono;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// A thread-safe context store for sharing state between components.
#[derive(Default)]
pub struct Context {
    store: RwLock<HashMap<String, Vec<u8>>>,
    history: RwLock<Vec<PromptRecord>>,
}

/// A record of a user prompt stored for adaptive history
#[derive(Clone)]
pub struct PromptRecord {
    /// The sanitized user prompt
    pub prompt: String,
    /// Unix timestamp when the prompt was recorded
    pub timestamp: u64,
}

impl Context {
    /// Creates a new, empty context.
    pub fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            history: RwLock::new(Vec::new()),
        }
    }

    /// Stores a value in the context.
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let serialized =
            bincode::serialize(value).map_err(|e| Error::SerializationError(e.to_string()))?;

        let mut store = self.store.write().map_err(|_| Error::LockError)?;

        store.insert(key.to_string(), serialized);
        Ok(())
    }

    /// Retrieves a value from the context.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let store = self.store.read().map_err(|_| Error::LockError)?;

        match store.get(key) {
            Some(bytes) => {
                let deserialized = bincode::deserialize(bytes)
                    .map_err(|e| Error::DeserializationError(e.to_string()))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    /// Removes a value from the context.
    pub fn remove(&self, key: &str) -> Result<()> {
        let mut store = self.store.write().map_err(|_| Error::LockError)?;

        store.remove(key);
        Ok(())
    }

    /// Checks if the context contains a key.
    pub fn contains_key(&self, key: &str) -> bool {
        self.store
            .read()
            .map(|store| store.contains_key(key))
            .unwrap_or(false)
    }

    /// Records a sanitized prompt in the history, keeping at most five entries.
    pub fn record_prompt(&self, prompt: &str) {
        if let Ok(mut hist) = self.history.write() {
            hist.push(PromptRecord {
                prompt: prompt.to_string(),
                timestamp: chrono::Utc::now().timestamp() as u64,
            });
            if hist.len() > 5 {
                hist.remove(0);
            }
        }
    }

    /// Returns the recent prompt history.
    pub fn history(&self) -> Vec<PromptRecord> {
        self.history.read().map(|h| h.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: i32,
        name: String,
    }

    #[test]
    fn test_context_storage() {
        let context = Context::new();
        let test_data = TestData {
            value: 42,
            name: "test".to_string(),
        };

        // Test storing and retrieving data
        context.set("test_key", &test_data).unwrap();
        let retrieved: TestData = context.get("test_key").unwrap().unwrap();

        assert_eq!(retrieved.value, 42);
        assert_eq!(retrieved.name, "test");

        // Test key existence
        assert!(context.contains_key("test_key"));

        // Test removal
        context.remove("test_key").unwrap();
        assert!(!context.contains_key("test_key"));
    }
}
