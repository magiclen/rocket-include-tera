use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    pub cache_table: Mutex<HashMap<Arc<str>, (Arc<str>, Arc<EntityTag>)>>,
}

/// To monitor the state of Tera.
#[cfg(not(debug_assertions))]
#[derive(Debug)]
pub struct TeraContextManager {
    pub tera: Tera,
    pub cache_table: Mutex<HashMap<Arc<str>, (Arc<str>, Arc<EntityTag>)>>,
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
    pub fn insert<S: Into<Arc<str>>>(&self, key: S, cache: (Arc<str>, Arc<EntityTag>)) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table.lock().unwrap().insert(key.into(), cache)
    }
}