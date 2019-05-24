#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_tera;

extern crate json_gettext;

use std::collections::HashMap;

use rocket::State;
use rocket_include_tera::{EtagIfNoneMatch, TeraResponse, TeraContextManager};

use json_gettext::JSONGetTextValue;

#[get("/")]
fn index() -> TeraResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    tera_response!("index", &map)
}

#[get("/etag")]
fn index_etag(etag_if_none_match: EtagIfNoneMatch) -> TeraResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    tera_response!(etag_if_none_match, "index", &map)
}

#[get("/2")]
fn index_2(cm: State<TeraContextManager>) -> TeraResponse {
    tera_response_static!(
        cm,
        "index2",
        {
            println!("Generate index_2 and staticize it...");

            let mut map = HashMap::new();

            map.insert("title", JSONGetTextValue::from_str("Title"));
            map.insert("placeholder", JSONGetTextValue::from_str("Hello, \"world!\""));
            map.insert("id", JSONGetTextValue::from_u64(0));

            tera_response!("index2", &map)
        }
    )
}

#[get("/2/etag")]
fn index_2_etag(etag_if_none_match: EtagIfNoneMatch, cm: State<TeraContextManager>) -> TeraResponse {
    tera_response_static!(
        etag_if_none_match,
        cm,
        "index2etag",
        {
            println!("Generate index_2_etag and staticize it...");

            let mut map = HashMap::new();

            map.insert("title", JSONGetTextValue::from_str("Title"));
            map.insert("placeholder", JSONGetTextValue::from_str("Hello, \"world!\""));
            map.insert("id", JSONGetTextValue::from_u64(0));

            tera_response!("index2", &map)
        }
    )
}

fn main() {
    rocket::ignite()
        .attach(TeraResponse::fairing(|tera| {
            tera_resources_initialize!(
                tera,
                "index", "examples/views/index.tera",
                "index2", "examples/views/index2.tera"
            );
        }))
        .mount("/", routes![index, index_etag])
        .mount("/", routes![index_2, index_2_etag])
        .launch();
}