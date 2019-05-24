use std::collections::HashMap;
use std::sync::Mutex;

use crate::{EntityTag};

#[cfg(debug_assertions)]
use crate::ReloadableTera;

#[cfg(not(debug_assertions))]
use crate::Tera;

/// To monitor the state of Tera.
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct TeraContextManager {
    pub tera: Mutex<ReloadableTera>,
    pub cache_table: Mutex<HashMap<String, (String, EntityTag)>>,
}

/// To monitor the state of Tera.
#[cfg(not(debug_assertions))]
#[derive(Debug)]
pub struct TeraContextManager {
    pub tera: Tera,
    pub cache_table: Mutex<HashMap<String, (String, EntityTag)>>,
}

impl TeraContextManager {
    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn new(tera: Mutex<ReloadableTera>) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_table: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn new(tera: Tera) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_table: Mutex::new(HashMap::new()),
        }
    }

    #[inline]
    /// Check if a cache key exists.
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.cache_table.lock().unwrap().contains_key(key.as_ref())
    }

    #[inline]
    /// Insert a cache.
    pub fn insert<S: Into<String>>(&self, key: S, cache: (String, EntityTag)) -> Option<(String, EntityTag)> {
        self.cache_table.lock().unwrap().insert(key.into(), cache)
    }
}