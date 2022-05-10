#![allow(unused_must_use)]

use rocket_include_tera::*;

#[test]
fn fairing_cache() {
    rocket::build().attach(tera_resources_initializer!(
        100;
        "index" => "examples/views/index.tera",
        "index2" => "examples/views/index2.tera"
    ));
}

#[test]
fn fairing() {
    rocket::build().attach(tera_resources_initializer!(
        "index" => "examples/views/index.tera",
        "index2" => "examples/views/index2.tera"
    ));
}
