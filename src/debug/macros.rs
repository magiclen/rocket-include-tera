/// Used in the fairing of `TeraResponse` to include Tera files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
#[macro_export]
macro_rules! tera_resources_initialize {
    ( $tera:expr, $($name:expr => $path:expr), * $(,)* ) => {
        {
            use ::std::fs;
            use ::std::collections::HashSet;

            let mut set: HashSet<&'static str> = HashSet::new();

            $(
                if set.contains($name) {
                    panic!("The name `{}` is duplicated.", $name);
                } else {
                    $tera.register_template_file($name, $crate::slash_formatter::concat_with_file_separator_debug_release!(env!("CARGO_MANIFEST_DIR"), $path)).unwrap();

                    set.insert($name);
                }
            )*
        }
    };
}

/// Used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
#[macro_export]
macro_rules! tera_response_cache {
    ($cm:expr, $etag_if_none_match:expr, $key:expr, $gen:block) => {{
        #[allow(unused_variables)]
        let __a = &$cm;
        #[allow(unused_variables)]
        let __a = &$key;

        let res = $gen;

        if res.weak_eq(&$etag_if_none_match) {
            $crate::TeraResponse::not_modified()
        } else {
            res
        }
    }};
}
