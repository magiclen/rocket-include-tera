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

use crate::{Tera, TeraContextManager};

const FAIRING_NAME: &'static str = "Tera";

/// The fairing of `TeraResponse`.
#[cfg(debug_assertions)]
pub struct TeraResponseFairing {
    pub(crate) custom_callback: Box<Fn(&mut MutexGuard<Tera>) + Send + Sync + 'static>
}

/// The fairing of `TeraResponse`.
#[cfg(not(debug_assertions))]
pub struct TeraResponseFairing {
    pub(crate) custom_callback: Box<Fn(&mut Tera) + Send + Sync + 'static>
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
        let tera = Mutex::new(Tera::default());

        (self.custom_callback)(&mut tera.lock().unwrap());

        let state = TeraContextManager::new(tera);

        Ok(rocket.manage(state))
    }

    #[cfg(not(debug_assertions))]
    fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
        let mut tera = Tera::default();

        tera.add_raw_template()

        (self.custom_callback)(&mut tera);

        let state = TeraContextManager::new(tera);

        Ok(rocket.manage(state))
    }

    #[cfg(debug_assertions)]
    fn on_request(&self, req: &mut Request, _data: &Data) {
        let cm = req.guard::<State<TeraContextManager>>().expect("TeraContextManager registered in on_attach");

        (self.custom_callback)(&mut cm.tera.lock().unwrap());
    }
}