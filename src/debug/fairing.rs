use std::sync::{Mutex, MutexGuard};

use crate::rocket::data::Data;
use crate::rocket::fairing::{Fairing, Info, Kind};
use crate::rocket::request::Request;
use crate::rocket::{Build, Rocket, State};

use super::{ReloadableTera, TeraContextManager, TeraResponse};

const FAIRING_NAME: &str = "Tera (Debug)";

/// The fairing of `TeraResponse`.
pub struct TeraResponseFairing {
    pub(crate) custom_callback:
        Box<dyn Fn(&mut MutexGuard<ReloadableTera>) -> usize + Send + Sync + 'static>,
}

#[rocket::async_trait]
impl Fairing for TeraResponseFairing {
    #[inline]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Ignite | Kind::Request,
        }
    }

    #[inline]
    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        let tera = Mutex::new(ReloadableTera::new());

        let cache_capacity = (self.custom_callback)(&mut tera.lock().unwrap());

        let state = TeraContextManager::new(tera, cache_capacity);

        Ok(rocket.manage(state))
    }

    #[inline]
    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data) {
        let cm = req
            .guard::<State<TeraContextManager>>()
            .await
            .expect("TeraContextManager registered in on_attach");

        cm.tera.lock().unwrap().reload_if_needed().unwrap();
    }
}

impl TeraResponse {
    /// Create the fairing of `TeraResponse`.
    #[inline]
    pub fn fairing<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut MutexGuard<ReloadableTera>) + Send + Sync + 'static, {
        let f = Box::new(f);

        TeraResponseFairing {
            custom_callback: Box::new(move |tera| {
                f(tera);

                crate::DEFAULT_CACHE_CAPACITY
            }),
        }
    }

    /// Create the fairing of `TeraResponse` and set the cache capacity.
    #[inline]
    pub fn fairing_cache<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut MutexGuard<ReloadableTera>) -> usize + Send + Sync + 'static, {
        TeraResponseFairing {
            custom_callback: Box::new(f),
        }
    }
}
