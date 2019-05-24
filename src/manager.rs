use std::collections::HashMap;
use std::sync::Mutex;

use crate::{EntityTag, Tera};

/// To monitor the state of Tera.
#[cfg(debug_assertions)]
#[derive(Debug)]
pub struct TeraContextManager {
    pub tera: Mutex<Tera>,
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
    pub(crate) fn new(tera: Mutex<Tera>) -> TeraContextManager {
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
}