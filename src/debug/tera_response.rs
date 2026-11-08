use std::io::Cursor;

use rocket::{
    http::Status,
    request::Request,
    response::{self, Responder, Response},
};

use crate::{EntityTag, EtagIfNoneMatch};

#[derive(Debug)]
enum TeraResponseInner {
    Html { content: String, etag: EntityTag<'static> },
    NotModified(Option<String>),
}

#[derive(Debug)]
/// To respond HTML.
pub struct TeraResponse {
    inner: TeraResponseInner,
}

impl TeraResponse {
    #[inline]
    pub(crate) fn build_not_cache<S: Into<String>>(
        content: S,
        etag: EntityTag<'static>,
    ) -> TeraResponse {
        TeraResponse {
            inner: TeraResponseInner::Html {
                content: content.into(),
                etag,
            },
        }
    }

    #[doc(hidden)]
    #[inline]
    pub const fn not_modified() -> TeraResponse {
        TeraResponse {
            inner: TeraResponseInner::NotModified(None)
        }
    }

    #[inline]
    pub(crate) fn not_modified_with_etag(etag: &EntityTag<'_>) -> Self {
        Self {
            inner: TeraResponseInner::NotModified(Some(etag.to_string()))
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn check_etag(self, etag_if_none_match: &EtagIfNoneMatch<'_>) -> Self {
        match &self.inner {
            TeraResponseInner::Html {
                etag, ..
            } if etag_if_none_match.weak_eq(etag) => Self::not_modified_with_etag(etag),
            _ => self,
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn weak_eq(&self, etag_if_none_match: &EtagIfNoneMatch<'_>) -> bool {
        matches!(&self.inner, TeraResponseInner::Html { etag, .. } if etag_if_none_match.weak_eq(etag))
    }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for TeraResponse {
    #[inline]
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'o> {
        let mut response = Response::build();

        match self.inner {
            TeraResponseInner::Html {
                content,
                etag,
            } => {
                response.raw_header("Content-Type", "text/html; charset=utf-8");
                response.raw_header(EntityTag::HEADER_NAME, etag.to_string());
                response.sized_body(content.len(), Cursor::new(content));
            },
            TeraResponseInner::NotModified(etag) => {
                response.status(Status::NotModified);

                if let Some(etag) = etag {
                    response.raw_header(EntityTag::HEADER_NAME, etag);
                }
            },
        }

        response.ok()
    }
}
