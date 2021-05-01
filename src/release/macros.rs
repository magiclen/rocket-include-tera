/// Used in the fairing of `TeraResponse` to include Tera files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
#[macro_export]
macro_rules! tera_resources_initialize {
    ( $tera:expr, $($name:expr => $path:expr), * $(,)* ) => {
        {
            use ::std::fs;
            use ::std::collections::HashSet;

            let mut set: HashSet<&str> = HashSet::new();

            $(
                if set.contains($name) {
                    panic!("The name `{}` is duplicated.", $name);
                } else {
                    $tera.add_raw_template($name, include_str!($crate::slash_formatter::concat_with_file_separator_debug_release!(env!("CARGO_MANIFEST_DIR"), $path))).unwrap();

                    set.insert($name);
                }
            )*
        }
    };
}

/// Used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
#[macro_export]
macro_rules! tera_response_cache {
    ($cm:expr, $etag_if_none_match:expr, $key:expr, $gen:block) => {
        match $cm.build_from_cache(&$etag_if_none_match, &$key) {
            Some(res) => res,
            None => {
                let res = $gen;

                match res.into_html_and_etag() {
                    Some((content, etag)) => {
                        let res = $crate::TeraResponse::build_cache(content.clone(), &etag);
                        $cm.insert($key, (content, ::std::sync::Arc::new(etag)));
                        res
                    }
                    None => $crate::TeraResponse::not_modified(),
                }
            }
        }
    };
}
