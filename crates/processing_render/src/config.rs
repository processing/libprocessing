//! Options object for configuring various aspects of libprocessing.
//!
//! To add a new Config just add a new enum with associated value

use bevy::prelude::Resource;
use std::collections::HashMap;

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum ConfigKey {
    AssetRootPath,
    SketchRootPath,
    SketchFileName,
}

// TODO: Consider Box<dyn Any> instead of String
#[derive(Resource)]
pub struct Config {
    map: HashMap<ConfigKey, String>,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            map: self.map.clone(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        // TODO consider defaults
        Config {
            map: HashMap::new(),
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
