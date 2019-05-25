/*!
# Include Tera Templates for Rocket Framework

This is a crate which provides macros `tera_resources_initialize!` and `tera_response!` to statically include Tera files from your Rust project and make them be the HTTP response sources quickly.

* `tera_resources_initialize!` is used for including Tera files into your executable binary file. You need to specify each file's name and its path. For instance, the above example uses **index** to represent the file **included-tera/index.tera** and **index-2** to represent the file **included-tera/index2.tera**. A name cannot be repeating. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `tera_response!` is used for retrieving and rendering the file you input through the macro `tera_resources_initialize!` as a `TeraResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
* `tera_response_static!` is used for in-memory staticizing a `TeraResponse` instance by a given key. It is effective only when you are using the **release** profile.

See `examples`.
*/

mod reloadable;
mod manager;
mod fairing;
mod macros;

pub extern crate tera;

extern crate crc_any;
extern crate html_minifier;

extern crate serde;

extern crate serde_json;

extern crate rocket;

extern crate rocket_etag_if_none_match;

use std::io::Cursor;
#[cfg(debug_assertions)]
use std::sync::MutexGuard;

use crc_any::CRC;
use tera::{Tera, Context, Error as TeraError};
use serde::Serialize;
use serde_json::{Value, Error as SerdeJsonError};

use rocket::State;
use rocket::request::Request;
use rocket::response::{self, Response, Responder};
use rocket::http::Status;
use rocket::fairing::Fairing;

pub use rocket_etag_if_none_match::{EntityTag, EtagIfNoneMatch};

pub use reloadable::ReloadableTera;
pub use manager::TeraContextManager;
use fairing::TeraResponseFairing;

#[inline]
fn compute_html_etag(html: &str) -> EntityTag {
    let mut crc64ecma = CRC::crc64ecma();
    crc64ecma.digest(html.as_bytes());
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
        etag: Option<EntityTag>,
        minify: bool,
        name: &'static str,
        context: Value,
    },
    Cache(String),
}

#[derive(Debug)]
/// To respond HTML from Tera templates.
pub struct TeraResponse {
    client_etag: EtagIfNoneMatch,
    source: TeraResponseSource,
}

impl TeraResponse {
    #[inline]
    /// Build a `TeraResponse` instance from a specific template.
    pub fn build_from_template<V: Serialize>(client_etag: EtagIfNoneMatch, etag: Option<EntityTag>, minify: bool, name: &'static str, context: V) -> Result<TeraResponse, SerdeJsonError> {
        let context = serde_json::to_value(context)?;

        let source = TeraResponseSource::Template {
            etag,
            minify,
            name,
            context,
        };

        Ok(TeraResponse {
            client_etag,
            source,
        })
    }

    #[inline]
    /// Build a `TeraResponse` instance from static cache.
    pub fn build_from_cache<S: Into<String>>(client_etag: EtagIfNoneMatch, name: S) -> TeraResponse {
        let source = TeraResponseSource::Cache(name.into());

        TeraResponse {
            client_etag,
            source,
        }
    }
}

impl TeraResponse {
    #[cfg(debug_assertions)]
    #[inline]
    /// Create the fairing of `TeraResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing where F: Fn(&mut MutexGuard<ReloadableTera>) + Send + Sync + 'static {
        TeraResponseFairing {
            custom_callback: Box::new(f)
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Create the fairing of `TeraResponse`.
    pub fn fairing<F>(f: F) -> impl Fairing where F: Fn(&mut Tera) + Send + Sync + 'static {
        TeraResponseFairing {
            custom_callback: Box::new(f)
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

                cm.tera.lock().unwrap().render(name, context)
            }
            _ => unreachable!()
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
            _ => unreachable!()
        }
    }

    #[cfg(debug_assertions)]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(&self, cm: &TeraContextManager) -> Result<(String, EntityTag), TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                let html = cm.tera.lock().unwrap().render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html, etag))
            }
            TeraResponseSource::Cache(name) => {
                let cache_table = cm.cache_table.lock().unwrap();

                match cache_table.get(name) {
                    Some((html, etag)) => Ok((html.clone(), etag.clone())),
                    None => Err(TeraError::msg("This Response hasn't triggered yet."))
                }
            }
        }
    }

    #[cfg(not(debug_assertions))]
    #[inline]
    /// Get this response's HTML and Etag.
    pub fn get_html_and_etag(&self, cm: &TeraContextManager) -> Result<(String, EntityTag), TeraError> {
        match &self.source {
            TeraResponseSource::Template {
                name,
                context,
                ..
            } => {
                let context = build_context(context);

                let html = cm.tera.render(name, context)?;

                let etag = compute_html_etag(&html);

                Ok((html, etag))
            }
            TeraResponseSource::Cache(name) => {
                let cache_table = cm.cache_table.lock().unwrap();

                match cache_table.get(name) {
                    Some((html, etag)) => Ok((html.clone(), etag.clone())),
                    None => Err(TeraError::msg("This Response hasn't triggered yet."))
                }
            }
        }
    }
}

impl<'a> Responder<'a> for TeraResponse {
    fn respond_to(self, request: &Request) -> response::Result<'a> {
        let mut response = Response::build();

        let cm = request.guard::<State<TeraContextManager>>().expect("TeraContextManager registered in on_attach");

        match &self.source {
            TeraResponseSource::Template {
                etag,
                minify,
                ..
            } => {
                let (html, etag) = match etag {
                    Some(etag) => {
                        let is_etag_match = self.client_etag.weak_eq(&etag);

                        if is_etag_match {
                            response.status(Status::NotModified);

                            return response.ok();
                        } else {
                            match self.render(&cm) {
                                Ok(html) => (html, etag.to_string()),
                                Err(_) => {
                                    response.status(Status::InternalServerError);

                                    return response.ok();
                                }
                            }
                        }
                    }
                    None => {
                        match self.render(&cm) {
                            Ok(html) => {
                                let etag = compute_html_etag(&html);

                                let is_etag_match = self.client_etag.weak_eq(&etag);

                                if is_etag_match {
                                    response.status(Status::NotModified);

                                    return response.ok();
                                } else {
                                    (html, etag.to_string())
                                }
                            }
                            Err(_) => {
                                response.status(Status::InternalServerError);

                                return response.ok();
                            }
                        }
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
                    let cache_table = cm.cache_table.lock().unwrap();

                    match cache_table.get(key) {
                        Some((html, etag)) => {
                            let is_etag_match = self.client_etag.weak_eq(etag);

                            if is_etag_match {
                                response.status(Status::NotModified);

                                return response.ok();
                            } else {
                                (html.clone(), etag.to_string())
                            }
                        }
                        None => {
                            response.status(Status::InternalServerError);

                            return response.ok();
                        }
                    }
                };

                response
                    .raw_header("ETag", etag)
                    .raw_header("Content-Type", "text/html; charset=utf-8")
                    .sized_body(Cursor::new(html));
            }
        }

        response.ok()
    }
}