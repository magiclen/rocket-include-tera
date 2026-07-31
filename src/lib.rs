/*!
# Include Tera Templates for Rocket Framework

This is a crate which provides macros `tera_resources_initialize!` and `tera_response!` to statically include Tera files from your Rust project and make them be the HTTP response sources quickly.

## Example

```rust,ignore
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
* `tera_response!` is used for retrieving and rendering the file you input through the macro `tera_resources_initialize!` as a `TeraResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
* `tera_response_cache!` is used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
* `tera_resources_initializer!` is used for generating a fairing for tera resources.
*/

#[macro_use]
extern crate educe;

#[doc(hidden)]
pub extern crate manifest_dir_macros;

mod functions;

#[cfg(debug_assertions)]
mod debug;

#[cfg(not(debug_assertions))]
mod release;

mod macros;

#[cfg(debug_assertions)]
pub use debug::*;
#[cfg(not(debug_assertions))]
pub use release::*;
pub use rocket_etag_if_none_match::{EtagIfNoneMatch, entity_tag::EntityTag};

const DEFAULT_CACHE_CAPACITY: usize = 64;
