Include Tera Templates for Rocket Framework
====================

[![Build Status](https://travis-ci.org/magiclen/rocket-include-tera.svg?branch=master)](https://travis-ci.org/magiclen/rocket-include-tera)

This is a crate which provides macros `tera_resources_initialize!` and `tera_response!` to statically include Tera files from your Rust project and make them be the HTTP response sources quickly.

* `tera_resources_initialize!` is used for including Tera files into your executable binary file. You need to specify each file's name and its path. For instance, the above example uses **index** to represent the file **included-tera/index.tera** and **index-2** to represent the file **included-tera/index2.tera**. A name cannot be repeating. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
* `tera_response!` is used for retrieving and rendering the file you input through the macro `tera_resources_initialize!` as a `TeraResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally be minified.
* `tera_response_static!` is used for in-memory staticizing a `TeraResponse` instance by a given key. It is effective only when you are using the **release** profile.

See `examples`.

## Crates.io

https://crates.io/crates/rocket-include-tera

## Documentation

https://docs.rs/rocket-include-tera

## License

[MIT](LICENSE)