#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_tera;

#[macro_use]
extern crate serde_json;

use std::collections::HashMap;

use rocket::State;

use rocket_include_tera::{EtagIfNoneMatch, TeraContextManager, TeraResponse};

#[get("/")]
fn index(tera_cm: State<TeraContextManager>, etag_if_none_match: EtagIfNoneMatch) -> TeraResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    tera_response!(tera_cm, etag_if_none_match, "index", map)
}

#[get("/disable-minify")]
fn index_disable_minify(
    tera_cm: State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch,
) -> TeraResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    tera_response!(disable_minify tera_cm, etag_if_none_match, "index", map)
}

#[get("/2")]
fn index_2(cm: State<TeraContextManager>, etag_if_none_match: EtagIfNoneMatch) -> TeraResponse {
    tera_response_cache!(cm, etag_if_none_match, "index-2", {
        println!("Generate index-2 and cache it...");

        let json = json! ({
            "title": "Title",
            "placeholder": "Hello, \"world!\"",
            "id": 0,
        });

        tera_response!(auto_minify cm, EtagIfNoneMatch::default(), "index2", json)
    })
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(TeraResponse::fairing(|tera| {
            tera_resources_initialize!(
                tera,
                "index" => "examples/views/index.tera",
                "index2" => "examples/views/index2.tera"
            );
        }))
        .mount("/", routes![index, index_disable_minify])
        .mount("/", routes![index_2])
}
