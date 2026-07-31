use std::{io::Cursor, sync::Arc};

use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use crate::EntityTag;

#[derive(Debug)]
enum TeraResponseInner {
    NotCache { content: String, etag: EntityTag<'static> },
    Cache { content: Arc<str>, etag: Arc<EntityTag<'static>> },
}

#[derive(Debug)]
/// To respond HTML.
pub struct TeraResponse {
    inner: Option<TeraResponseInner>,
}

impl TeraResponse {
    #[inline]
    pub(crate) fn build_not_cache<S: Into<String>>(
        content: S,
        etag: EntityTag<'static>,
    ) -> TeraResponse {
        TeraResponse {
            inner: Some(TeraResponseInner::NotCache {
                content: content.into(),
                etag,
            }),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn build_cache(content: Arc<str>, etag: Arc<EntityTag<'static>>) -> TeraResponse {
        TeraResponse {
            inner: Some(TeraResponseInner::Cache {
                content,
                etag,
            }),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub const fn not_modified() -> TeraResponse {
        TeraResponse {
            inner: None
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn into_html_and_etag(self) -> Option<(Arc<str>, Arc<EntityTag<'static>>)> {
        match self.inner {
            Some(TeraResponseInner::NotCache {
                content,
                etag,
            }) => Some((Arc::from(content), Arc::new(etag))),
            Some(TeraResponseInner::Cache {
                content,
                etag,
            }) => Some((content, etag)),
            None => None,
        }
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for TeraResponse {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        // `Arc<str>` does not implement `AsRef<[u8]>`, so wrap it to let `Cursor` read the cached HTML without copying it.
        #[derive(Debug)]
        struct SharedHtml(Arc<str>);

        impl AsRef<[u8]> for SharedHtml {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                self.0.as_bytes()
            }
        }

        let mut response = Response::build();

        if let Some(inner) = self.inner {
            response.raw_header("Content-Type", "text/html; charset=utf-8");

            match inner {
                TeraResponseInner::NotCache {
                    content,
                    etag,
                } => {
                    response.raw_header("Etag", etag.to_string());
                    response.sized_body(content.len(), Cursor::new(content));
                },
                TeraResponseInner::Cache {
                    content,
                    etag,
                } => {
                    response.raw_header("Etag", etag.to_string());
                    response.sized_body(content.len(), Cursor::new(SharedHtml(content)));
                },
            }
        } else {
            response.status(Status::NotModified);
        }

        response.ok()
    }
}
