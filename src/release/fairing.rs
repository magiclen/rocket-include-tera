use rocket::fairing::{Fairing, Info, Kind};
use rocket::{Build, Rocket};

use tera::Tera;

use super::{TeraContextManager, TeraResponse};

const FAIRING_NAME: &str = "Tera";

/// The fairing of `TeraResponse`.
pub struct TeraResponseFairing {
    pub(crate) custom_callback: Box<dyn Fn(&mut Tera) -> usize + Send + Sync + 'static>,
}

#[rocket::async_trait]
impl Fairing for TeraResponseFairing {
    #[inline]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Ignite,
        }
    }

    #[inline]
    async fn on_ignite(&self, rocket: Rocket<Build>) -> Result<Rocket<Build>, Rocket<Build>> {
        let mut tera = Tera::default();

        let cache_capacity = (self.custom_callback)(&mut tera);

        let state = TeraContextManager::new(tera, cache_capacity);

        Ok(rocket.manage(state))
    }
}

impl TeraResponse {
    /// Create the fairing of `TeraResponse`.
    #[inline]
    pub fn fairing<F>(f: F) -> impl Fairing
    where
        F: Fn(&mut Tera) + Send + Sync + 'static, {
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
        F: Fn(&mut Tera) -> usize + Send + Sync + 'static, {
        TeraResponseFairing {
            custom_callback: Box::new(f),
        }
    }
}
