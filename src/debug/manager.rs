use std::sync::{Mutex, PoisonError};

use serde::Serialize;
use tera::Context;

use super::{ReloadableTera, TeraResponse};
use crate::{functions::compute_data_etag, EtagIfNoneMatch};

/// To monitor the state of Tera.
#[derive(Educe)]
#[educe(Debug)]
pub struct TeraContextManager {
    pub tera: Mutex<ReloadableTera>,
}

impl TeraContextManager {
    #[inline]
    pub(crate) fn new(tera: Mutex<ReloadableTera>, _cache_capacity: usize) -> TeraContextManager {
        TeraContextManager {
            tera,
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
        self.tera
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .render(name.as_ref(), &Context::from_serialize(context).unwrap())
            .map(|html| {
                let etag = compute_data_etag(html.as_bytes());

                if etag_if_none_match.weak_eq(&etag) {
                    TeraResponse::not_modified()
                } else {
                    let html = if minify { html_minifier::minify(html).unwrap() } else { html };

                    TeraResponse::build_not_cache(html, &etag)
                }
            })
            .unwrap()
    }

    /// Render a template.
    #[inline]
    pub fn render<S: AsRef<str>, V: Serialize>(&self, name: S, context: V) -> String {
        self.tera
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .render(name.as_ref(), &Context::from_serialize(context).unwrap())
            .unwrap()
    }
}
