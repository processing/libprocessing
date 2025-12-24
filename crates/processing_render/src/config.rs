//! Options object for configuring various aspects of libprocessing.
//!
//! To add a new Config just add a new enum with associated value

use std::collections::HashMap;

#[derive(Hash, Eq, PartialEq)]
pub enum ConfigKey {
    AssetRootPath,
}
// TODO: Consider Box<dyn Any> instead of String
pub type ConfigMap = HashMap<ConfigKey, String>;
pub struct Config {
    map: ConfigMap,
}

impl Config {
    pub fn new() -> Self {
        // TODO consider defaults
        Config {
            map: ConfigMap::new(),
        }
    }

    pub fn get(&self, k: ConfigKey) -> Option<&String> {
        self.map.get(&k)
    }

    pub fn set(&mut self, k: ConfigKey, v: String) {
        self.map.insert(k, v);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
