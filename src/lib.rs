/*!
# Include Tera Templates for Rocket Framework

This is a crate which provides macros `tera_resources_initialize!` and `tera_response!` to statically include Tera files from your Rust project and make them be the HTTP response sources quickly.

* `tera_resources_initialize!` is used in the fairing of `TeraResponseFairing` to include Tera files into your executable binary file. You need to specify each file's name and its path. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `tera_response!` is used for retrieving and rendering the file you input through the macro `tera_resources_initialize!` as a `TeraResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
* `tera_response_cache!` is used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.

See `examples`.
*/

mod fairing;
mod macros;
mod manager;
mod reloadable;

pub extern crate tera;

#[macro_use]
extern crate educe;
extern crate crc_any;
extern crate html_minifier;
extern crate lru_time_cache;
extern crate rc_u8_reader;

extern crate serde;

extern crate serde_json;

extern crate rocket;

extern crate rocket_etag_if_none_match;

use std::io::Cursor;
use std::sync::Arc;
#[cfg(debug_assertions)]
use std::sync::MutexGuard;

use crc_any::CRCu64;
use rc_u8_reader::ArcU8Reader;
use serde::Serialize;
use serde_json::{Error as SerdeJsonError, Value};
use tera::{Context, Error as TeraError, Tera};

use rocket::fairing::Fairing;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::State;

use rocket_etag_if_none_match::{EntityTag, EtagIfNoneMatch};

use fairing::TeraResponseFairing;
pub use manager::TeraContextManager;
pub use reloadable::ReloadableTera;

const DEFAULT_CACHE_CAPACITY: usize = 64;

#[inline]
fn compute_html_etag<S: AsRef<str>>(html: S) -> EntityTag {
    let mut crc64ecma = CRCu64::crc64();
    crc64ecma.digest(html.as_ref().as_bytes());
    let crc64 = crc64ecma.get_crc();
    EntityTag::new(true, format!("{:X}", crc64))
}

#[inline]
fn build_context(value: &Value) -> Context {
    let mut context = Context::new();

    if let Value::Object(map) = value {
        for (k, v) in map {
            context.insert(k, v);
        }
    }

    context
}

#[derive(Debug)]
enum TeraResponseSource {
    Template {
        minify: bool,
        name: &'static str,
        context: Value,
    },
    Cache(Arc<str>),
}

#[derive(Debug)]
/// To respond HTML from Tera templates.
pub struct TeraResponse {
    source: TeraResponseSource,
}

impl TeraResponse {
    #[inline]
    /// Build a `TeraResponse` instance from a specific template.
    pub fn build_from_template<V: Serialize>(
        minify: bool,
        name: &'static str,
        context: V,
    ) -> Result<TeraResponse, SerdeJsonError> {
        let context = serde_json::to_value(context)?;

        let source = TeraResponseSource::Template {
            minify,
            name,
            context,
        };

        Ok(TeraResponse {
            source,
        })
    }

    #[inline]
    /// Build a `TeraResponse` instance from cache.
    pub fn build_from_cache<S: Into<Arc<str>>>(name: S) -> TeraResponse {
        let source = TeraResponseSource::Cache(name.into());

        TeraResponse {
            source,
        }
    }
}

impl TeraResponse {
    #[cfg(debug_assertions)]
    #[inline]
    /// Create the fairing of `TeraResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut MutexGuard<ReloadableTera>) + Send + Sync + 'static, {
        let f = Box::new(f);

        TeraResponseFairing {
            custom_callback: Box::new(move |tera| {
                f(tera);

                DEFAULT_CACHE_CAPACITY
            }),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Create the fairing of `TeraResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut Tera) + Send + Sync + 'static, {
        let f = Box::new(f);

        TeraResponseFairing {
            custom_callback: Box::new(move |tera| {
                f(tera);

                DEFAULT_CACHE_CAPACITY
            }),
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Create the fairing of `TeraResponse`.
    pub fn fairing_cache<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut MutexGuard<ReloadableTera>) -> usize + Send + Sync + 'static, {
        TeraResponseFairing {
            custom_callback: Box::new(f),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Create the fairing of `TeraResponse`.
    pub fn fairing_cache<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut Tera) -> usize + Send + Sync + 'static, {
        TeraResponseFairing {
            custom_callback: Box::new(f),
        }
    }
}

impl TeraResponse {
    #[cfg(debug_assertions)]
    #[inline]
    fn render(&self, cm: &TeraContextManager) -> Result<String, TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                cm.tera.lock().unwrap().render(name, &context)
            }
            _ => unreachable!(),
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    fn render(&self, cm: &TeraContextManager) -> Result<String, TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                cm.tera.render(name, context)
            }
            _ => unreachable!(),
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(
        &self,
        cm: &TeraContextManager,
    ) -> Result<(Arc<str>, Arc<EntityTag>), TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                let html = cm.tera.lock().unwrap().render(name, &context)?;

                let etag = compute_html_etag(&html);

                Ok((html.into(), Arc::new(etag)))
            }
            TeraResponseSource::Cache(key) => {
                cm.get(key)
                    .ok_or_else(|| TeraError::msg("This response hasn't been triggered yet."))
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(
        &self,
        cm: &TeraContextManager,
    ) -> Result<(Arc<str>, Arc<EntityTag>), TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                let html = cm.tera.render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html.into(), Arc::new(etag)))
            }
            TeraResponseSource::Cache(key) => {
                cm.get(key).ok_or(TeraError::msg("This response hasn't been triggered yet."))
            }
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Get this response's HTML.
    pub fn get_html(&self, cm: &TeraContextManager) -> Result<String, TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                let html = cm.tera.lock().unwrap().render(name, &context)?;

                Ok(html)
            }
            TeraResponseSource::Cache(key) => {
                cm.get(key)
                    .map(|(html, _)| html.to_string())
                    .ok_or_else(|| TeraError::msg("This response hasn't been triggered yet."))
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Get this response's HTML.
    pub fn get_html(&self, cm: &TeraContextManager) -> Result<String, TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                let html = cm.tera.render(name, context)?;

                Ok(html)
            }
            TeraResponseSource::Cache(key) => {
                cm.get(key)
                    .map(|(html, _)| html.to_string())
                    .ok_or(TeraError::msg("This response hasn't been triggered yet."))
            }
        }
    }
}

impl<'a> Responder<'a> for TeraResponse {
    fn respond_to(self, request: &Request) -> response::Result<'a> {
        let client_etag = request.guard::<EtagIfNoneMatch>().unwrap();

        let mut response = Response::build();

        let cm = request
            .guard::<State<TeraContextManager>>()
            .expect("TeraContextManager registered in on_attach");

        match &self.source {
            TeraResponseSource::Template {
                minify,
                ..
            } => {
                let (html, etag) = match self.render(&cm) {
                    Ok(html) => {
                        let etag = compute_html_etag(&html);

                        let is_etag_match = client_etag.weak_eq(&etag);

                        if is_etag_match {
                            response.status(Status::NotModified);

                            return response.ok();
                        } else {
                            (html, etag.to_string())
                        }
                    }
                    Err(_) => {
                        return Err(Status::InternalServerError);
                    }
                };

                let html = if *minify {
                    html_minifier::minify(&html).unwrap()
                } else {
                    html
                };

                response
                    .raw_header("ETag", etag)
                    .raw_header("Content-Type", "text/html; charset=utf-8")
                    .sized_body(Cursor::new(html));
            }
            TeraResponseSource::Cache(key) => {
                let (html, etag) = {
                    match cm.get(key) {
                        Some((html, etag)) => {
                            let is_etag_match = client_etag.weak_eq(&etag);

                            if is_etag_match {
                                response.status(Status::NotModified);

                                return response.ok();
                            } else {
                                (html, etag.to_string())
                            }
                        }
                        None => {
                            return Err(Status::InternalServerError);
                        }
                    }
                };

                response
                    .raw_header("ETag", etag)
                    .raw_header("Content-Type", "text/html; charset=utf-8")
                    .sized_body(ArcU8Reader::new(html));
            }
        }

        response.ok()
    }
}
