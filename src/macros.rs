/// Used for retrieving and rendering the file you input through the macro `tera_resources_initialize!` as a `TeraResponse` instance with rendered HTML. When its `respond_to` method is called, three HTTP headers, **Content-Type**, **Content-Length** and **Etag**, will be automatically added, and the rendered HTML can optionally not be minified.
#[macro_export]
macro_rules! tera_response {
    ( $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::tera_response!($cm, $etag_if_none_match, $name, map)
        }
    };
    ( $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $crate::tera_response!(enable_minify $cm, $etag_if_none_match, $name, $data)
    };
    ( enable_minify $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::tera_response!(enable_minify $cm, $etag_if_none_match, $name, map)
        }
    };
    ( enable_minify $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $cm.build(
            &$etag_if_none_match,
            true,
            $name,
            &$data,
        )
    };
    ( disable_minify $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::tera_response!(disable_minify $cm, $etag_if_none_match, $name, map)
        }
    };
    ( disable_minify $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        $cm.build(
            &$etag_if_none_match,
            false,
            $name,
            &$data,
        )
    };
    ( auto_minify $cm:expr, $etag_if_none_match:expr, $name:expr ) => {
        {
            use ::std::collections::HashMap;

            let map: HashMap<u8, u8> = HashMap::new();

            $crate::tera_response!(auto_minify $cm, $etag_if_none_match, $name, map)
        }
    };
    ( auto_minify $cm:expr, $etag_if_none_match:expr, $name:expr, $data:expr ) => {
        if cfg!(debug_assertions) {
            tera_response!(disable_minify $cm, $etag_if_none_match, $name, $data)
        } else {
            tera_response!(enable_minify $cm, $etag_if_none_match, $name, $data)
        }
    };
}

/// Used for generating a fairing for tera resources.
#[macro_export]
macro_rules! tera_resources_initializer {
    ( $($name:expr => $path:expr), * $(,)* ) => {
        {
            $crate::TeraResponse::fairing(|tera| {
                $crate::tera_resources_initialize!(
                    tera
                    $(, $name => $path)*
                );
            })
        }
    };
    ( $capacity:expr; $($name:expr => $path:expr), * $(,)*  ) => {
        {
            $crate::TeraResponse::fairing_cache(|tera| {
                $crate::tera_resources_initialize!(
                    tera
                    $(, $name => $path)*
                );

                $capacity
            })
        }
    };
}
