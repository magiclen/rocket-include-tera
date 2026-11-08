use std::sync::{Arc, Mutex, PoisonError};

use lru_time_cache::LruCache;
use serde::Serialize;
use tera::{Context, Tera};

use super::TeraResponse;
use crate::{EntityTag, EtagIfNoneMatch, functions::compute_data_etag};

#[allow(clippy::type_complexity)]
/// To monitor the state of Tera.
#[derive(Educe)]
#[educe(Debug)]
pub struct TeraContextManager {
    pub tera:    Tera,
    #[educe(Debug(ignore))]
    cache_table: Option<Mutex<LruCache<String, (Arc<str>, Arc<EntityTag<'static>>)>>>,
}

impl TeraContextManager {
    #[inline]
    pub(crate) fn new(tera: Tera, cache_capacity: usize) -> TeraContextManager {
        TeraContextManager {
            tera,
            cache_table: (cache_capacity > 0)
                .then(|| Mutex::new(LruCache::with_capacity(cache_capacity))),
        }
    }

    /// Build a `TeraResponse`.
    #[inline]
    pub fn build<S: AsRef<str>, V: Serialize>(
        &self,
        etag_if_none_match: &EtagIfNoneMatch<'_>,
        minify: bool,
        name: S,
        context: V,
    ) -> TeraResponse {
        let html = self.render(name, context);
        let etag = compute_data_etag(html.as_bytes());

        if etag_if_none_match.weak_eq(&etag) {
            return TeraResponse::not_modified_with_etag(&etag);
        }

        let html = if minify { html_minifier::minify(&html).unwrap_or(html) } else { html };

        TeraResponse::build_not_cache(html, etag)
    }

    /// Build a `TeraResponse` from the cache.
    /// Return `None` if the key is not cached or caching is disabled.
    #[inline]
    pub fn build_from_cache<K: AsRef<str>>(
        &self,
        etag_if_none_match: &EtagIfNoneMatch<'_>,
        key: K,
    ) -> Option<TeraResponse> {
        let mut cache = self.cache_table.as_ref()?.lock().unwrap_or_else(PoisonError::into_inner);
        let (html, etag) = cache.get(key.as_ref())?;

        if etag_if_none_match.weak_eq(etag) {
            let etag = etag.clone();

            drop(cache);

            Some(TeraResponse::not_modified_with_etag(&etag))
        } else {
            Some(TeraResponse::build_cache(html.clone(), etag.clone()))
        }
    }

    /// Render a template.
    #[inline]
    pub fn render<S: AsRef<str>, V: Serialize>(&self, name: S, context: V) -> String {
        self.tera.render(name.as_ref(), &Context::from_serialize(&context).unwrap()).unwrap()
    }

    /// Clear cache.
    #[inline]
    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache_table {
            cache.lock().unwrap_or_else(PoisonError::into_inner).clear();
        }
    }

    /// Check if a cache key exists.
    #[inline]
    pub fn contains_key<S: AsRef<str>>(&self, key: S) -> bool {
        // `contains_key` is used instead of `get` because merely asking about a key should not make it the most recently used one.
        self.cache_table.as_ref().is_some_and(|cache| {
            cache.lock().unwrap_or_else(PoisonError::into_inner).contains_key(key.as_ref())
        })
    }

    /// Get the cache by a specific key.
    #[inline]
    pub fn get<S: AsRef<str>>(&self, key: S) -> Option<(Arc<str>, Arc<EntityTag<'static>>)> {
        self.cache_table
            .as_ref()?
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(key.as_ref())
            .map(|(html, etag)| (html.clone(), etag.clone()))
    }

    /// Insert a cache.
    #[inline]
    pub fn insert<S: Into<String>>(
        &self,
        key: S,
        cache: (Arc<str>, Arc<EntityTag<'static>>),
    ) -> Option<(Arc<str>, Arc<EntityTag<'static>>)> {
        self.cache_table
            .as_ref()?
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.into(), cache)
    }
}
