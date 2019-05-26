use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::EntityTag;

#[cfg(debug_assertions)]
use crate::ReloadableTera;

#[cfg(not(debug_assertions))]
use crate::Tera;

/// To monitor the state of Tera.
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct TeraContextManager {
    pub tera: Mutex<ReloadableTera>,
    cache_capacity: usize,
    pub cache_table: Mutex<(Vec<Arc<str>>, HashMap<Arc<str>, (Arc<str>, Arc<EntityTag>)>)>,
}

/// To monitor the state of Tera.
#[cfg(not(debug_assertions))]
#[derive(Debug)]
pub struct TeraContextManager {
    pub tera: Tera,
    cache_capacity: usize,
    pub cache_table: Mutex<(Vec<Arc<str>>, HashMap<Arc<str>, (Arc<str>, Arc<EntityTag>)>)>,
}

impl TeraContextManager {
    #[cfg(debug_assertions)]
    #[inline]
    pub(crate) fn new(tera: Mutex<ReloadableTera>, cache_capacity: usize) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_capacity,
            cache_table: Mutex::new((Vec::new(), HashMap::new())),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    pub(crate) fn new(tera: Tera, cache_capacity: usize) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_capacity,
            cache_table: Mutex::new((Vec::new(), HashMap::new())),
        }
    }

    #[inline]
    /// Get the capacity of this cache.
    pub fn get_cache_capacity(&self) -> usize {
        self.cache_capacity
    }

    #[inline]
    /// Get the size of this cache.
    pub fn get_cache_size(&self) -> usize {
        self.cache_table.lock().unwrap().0.len()
    }

    #[inline]
    /// Clear cache.
    pub fn clear_cache(&self) {
        let mut cache_table = self.cache_table.lock().unwrap();

        cache_table.0.clear();
        cache_table.1.clear();
    }

    #[inline]
    /// Check if a cache key exists.
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        self.cache_table.lock().unwrap().1.contains_key(key.as_ref())
    }

    #[inline]
    /// Check if a cache key exists.
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<(Arc<str>, Arc<EntityTag>)> {
        self.cache_table.lock().unwrap().1.get(key.as_ref()).map(|(html, etag)| (html.clone(), etag.clone()))
    }

    #[inline]
    /// Insert a cache.
    pub fn insert<S: Into<Arc<str>>>(&self, key: S, cache: (Arc<str>, Arc<EntityTag>)) -> Option<(Arc<str>, Arc<EntityTag>)> {
        if self.cache_capacity == 0 {
            None
        } else {
            let mut cache_table = self.cache_table.lock().unwrap();

            let key: Arc<str> = key.into();

            if let Some(index) = cache_table.0.iter().rposition(|v| key.eq(&v)) {
                let key_2 = cache_table.0.remove(index);

                cache_table.0.push(key_2);

                cache_table.1.insert(key, cache)
            } else {
                let size = cache_table.0.len();

                if size == self.cache_capacity {
                    let key = cache_table.0.pop().unwrap();

                    cache_table.1.remove(&key);
                }

                cache_table.0.push(key.clone());
                cache_table.1.insert(key, cache)
            }
        }
    }
}