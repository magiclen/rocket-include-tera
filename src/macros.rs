/// Used in the fairing of `TeraResponseFairing` to include Tera files into your executable binary file. You need to specify each file's name and its path. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! tera_resources_initialize {
    ( $tera:expr, $($name:expr, $path:expr), * $(,)* ) => {
        use std::fs;

        $(
            $tera.register_template_file($name, $path).unwrap();
        )*
    };
}

/// Used in the fairing of `TeraResponseFairing` to include Tera files into your executable binary file. You need to specify each file's name and its path. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! tera_resources_initialize {
    ( $tera:expr, $($name:expr, $path:expr), * $(,)* ) => {
        use std::fs;

        $(
            $tera.add_raw_template($name, include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", $path))).unwrap();
        )*
    };
}

/// Used for retrieving and rendering the file you input through the macro `tera_resources_initialize!` as a `TeraResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
#[macro_export]
macro_rules! tera_response {
    ( $name:expr, $data:expr ) => {
        {
            use ::rocket_include_tera::{TeraResponse, EtagIfNoneMatch};

            TeraResponse::build_from_template(
                EtagIfNoneMatch {
                    etag: None
                },
                true,
                $name,
                $data,
            ).unwrap()
        }
    };
    ( disable_minify $name:expr, $data:expr ) => {
        {
            use ::rocket_include_tera::{TeraResponse, EtagIfNoneMatch};

            TeraResponse::build_from_template(
                EtagIfNoneMatch {
                    etag: None
                },
                false,
                $name,
                $data,
            ).unwrap()
        }
    };
    ( $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        {
            use ::rocket_include_tera::{TeraResponse, EtagIfNoneMatch};

            TeraResponse::build_from_template(
                $etag_if_none_match,
                true,
                $name,
                $data,
            ).unwrap()
        }
    };
    ( disable_minify $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        {
            use ::rocket_include_tera::{TeraResponse, EtagIfNoneMatch};

            TeraResponse::build_from_template(
                $etag_if_none_match,
                false,
                $name,
                $data,
            ).unwrap()
        }
    };
    ( $etag_if_none_match:expr, disable_minify $name:expr, $data:expr ) => {
        {
            use ::rocket_include_tera::{TeraResponse, EtagIfNoneMatch};

            TeraResponse::build_from_template(
                $etag_if_none_match,
                false,
                $name,
                $data,
            ).unwrap()
        }
    };
}

/// Used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
#[macro_export]
#[cfg(debug_assertions)]
macro_rules! tera_response_cache {
    ( $cm:expr, $key:expr, $gen:block ) => {
        {
            drop(&$cm);
            drop(&$key);
            $gen
        }
    };
    ( $etag_if_none_match:expr, $cm:expr, $key:expr, $gen:block ) => {
        {
            drop(&$etag_if_none_match);
            drop(&$cm);
            drop(&$key);
            $gen
        }
    };
}

/// Used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
#[macro_export]
#[cfg(not(debug_assertions))]
macro_rules! tera_response_cache {
    ( $cm:expr, $key:expr, $gen:block ) => {
        {
            let contains = $cm.contains_key($key);

            if contains {
                TeraResponse::build_from_cache(
                    EtagIfNoneMatch {
                        etag: None
                    },
                    $key
                )
            } else {
                let res = $gen;

                let cache = res.get_html_and_etag(&$cm).unwrap();

                $cm.insert($key, cache);

                res
            }
        }
    };
    ( $etag_if_none_match:expr, $cm:expr, $key:expr, $gen:block ) => {
        {
            let contains = $cm.contains_key($key);

            if contains {
                TeraResponse::build_from_cache(
                    $etag_if_none_match,
                    $key
                )
            } else {
                let res = $gen;

                let cache = res.get_html_and_etag(&$cm).unwrap();

                $cm.insert($key, cache);

                res
            }
        }
    };
}