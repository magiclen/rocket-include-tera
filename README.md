Include Tera Templates for Rocket Framework
====================

[![CI](https://github.com/magiclen/rocket-include-tera/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/rocket-include-tera/actions/workflows/ci.yml)

This is a crate which provides macros `tera_resources_initialize!` and `tera_response!` to statically include Tera files from your Rust project and make them be the HTTP response sources quickly.

## Example

```rust
#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_include_tera;

use std::collections::HashMap;

use rocket::State;

use rocket_include_tera::{EtagIfNoneMatch, TeraContextManager, TeraResponse};

#[get("/")]
fn index(tera_cm: &State<TeraContextManager>, etag_if_none_match: EtagIfNoneMatch) -> TeraResponse {
    let mut map = HashMap::new();

    map.insert("title", "Title");
    map.insert("body", "Hello, world!");

    tera_response!(tera_cm, etag_if_none_match, "index", map)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(tera_resources_initializer!(
            "index" => "views/index.tera",
        ))
        .mount("/", routes![index])
}
```

* `tera_resources_initialize!` is used in the fairing of `TeraResponse` to include Tera files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `tera_response!` renders a registered template as a `TeraResponse`.
  A `200 OK` response includes `Content-Type`, `Content-Length`, and `ETag`.
  A `304 Not Modified` response includes the `ETag` and has no body.
  You can turn off HTML minification.
* `tera_response_cache!` is used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
* `tera_resources_initializer!` is used for generating a fairing for tera resources.

## Crates.io

https://crates.io/crates/rocket-include-tera

## Documentation

https://docs.rs/rocket-include-tera

## License

[MIT](LICENSE)
