use std::sync::{Arc, Mutex};

use crate::EntityTag;

#[cfg(debug_assertions)]
use crate::ReloadableTera;

#[cfg(not(debug_assertions))]
use crate::Tera;

use crate::lru_time_cache::LruCache;

/// To monitor the state of Tera.
#[cfg(debug_assertions)]
#[derive(Educe)]
#[educe(Debug)]
pub struct TeraContextManager {
    pub tera: Mutex<ReloadableTera>,
    #[educe(Debug(ignore))]
    cache_table: Mutex<LruCache<Arc<str>, (Arc<str>, Arc<EntityTag>)>>,
}

/// To monitor the state of Tera.
#[cfg(not(debug_assertions))]
#[derive(Educe)]
#[educe(Debug)]
pub struct TeraContextManager {
    pub tera: Tera,
    #[educe(Debug(ignore))]
    cache_table: Mutex<LruCache<Arc<str>, (Arc<str>, Arc<EntityTag>)>>,
}

impl TeraContextManager {
    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn new(tera: Mutex<ReloadableTera>, cache_capacity: usize) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_table: Mutex::new(LruCache::with_capacity(cache_capacity)),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn new(tera: Tera, cache_capacity: usize) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_table: Mutex::new(LruCache::with_capacity(cache_capacity)),
        }
    }

    #[inline]
    /// Clear cache.
    pub fn clear_cache(&self) {
        self.cache_table.lock().unwrap().clear();
    }

    #[inline]
    /// Check if a cache key exists.
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.cache_table.lock().unwrap().get(key.as_ref()).is_some()
    }

    #[inline]
    /// Get the cache by a specific key.
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table.lock().unwrap().get(key.as_ref()).map(|(html, etag)| (html.clone(), etag.clone()))
    }

    #[inline]
    /// Insert a cache.
    pub fn insert<S: Into<Arc<str>>>(&self, key: S, cache: (Arc<str>, Arc<EntityTag>)) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table.lock().unwrap().insert(key.into(), cache)
    }
}