#[cfg(debug_assertions)]
use std::sync::{Mutex, MutexGuard};

use crate::rocket::Rocket;
#[cfg(debug_assertions)]
use crate::rocket::State;
#[cfg(debug_assertions)]
use crate::rocket::request::Request;
use crate::rocket::fairing::{Fairing, Info, Kind};
#[cfg(debug_assertions)]
use crate::rocket::data::Data;

#[cfg(debug_assertions)]
use crate::ReloadableTera;

#[cfg(not(debug_assertions))]
use crate::Tera;

use crate::TeraContextManager;

const FAIRING_NAME: &'static str = "Tera";

/// The fairing of `TeraResponse`.
#[cfg(debug_assertions)]
pub struct TeraResponseFairing {
    pub(crate) custom_callback: Box<dyn Fn(&mut MutexGuard<ReloadableTera>) -> usize + Send + Sync + 'static>,
}

/// The fairing of `TeraResponse`.
#[cfg(not(debug_assertions))]
pub struct TeraResponseFairing {
    pub(crate) custom_callback: Box<dyn Fn(&mut Tera) -> usize + Send + Sync + 'static>,
}

impl Fairing for TeraResponseFairing {
    #[cfg(debug_assertions)]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Attach | Kind::Request,
        }
    }

    #[cfg(not(debug_assertions))]
    fn info(&self) -> Info {
        Info {
            name: FAIRING_NAME,
            kind: Kind::Attach,
        }
    }

    #[cfg(debug_assertions)]
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let tera = Mutex::new(ReloadableTera::new());

        let cache_capacity = (self.custom_callback)(&mut tera.lock().unwrap());

        let state = TeraContextManager::new(tera, cache_capacity);

        Ok(rocket.manage(state))
    }

    #[cfg(not(debug_assertions))]
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let mut tera = Tera::default();

        let cache_capacity = (self.custom_callback)(&mut tera);

        let state = TeraContextManager::new(tera, cache_capacity);

        Ok(rocket.manage(state))
    }

    #[cfg(debug_assertions)]
    fn on_request(&self, req: &mut Request, _data: &Data) {
        let cm = req.guard::<State<TeraContextManager>>().expect("TeraContextManager registered in on_attach");

        cm.tera.lock().unwrap().reload_if_needed().unwrap();
    }
}