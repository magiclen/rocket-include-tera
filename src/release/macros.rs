/// Used in the fairing of `TeraResponse` to include Tera files into your executable binary file. You need to specify each file's name and its path relative to the directory containing the manifest of your package. In order to reduce the compilation time and allow to hot-reload templates, files are compiled into your executable binary file together, only when you are using the **release** profile.
#[macro_export]
macro_rules! tera_resources_initialize {
    ( $tera:expr, $(,)* ) => {{}};
    ( $tera:expr, $($name:expr => $path:expr),+ $(,)* ) => {{
        let mut names = ::std::collections::HashSet::new();
        let templates = [$({
            let name: &'static str = $name;

            if !names.insert(name) {
                panic!("The name `{}` is duplicated.", name);
            }

            (name, include_str!($crate::manifest_dir_macros::path!($path)))
        }),+];

        $tera.add_raw_templates(templates).unwrap();
    }};
}

/// Used for wrapping a `TeraResponse` and its constructor, and use a **key** to cache its HTML and ETag in memory. The cache is generated only when you are using the **release** profile.
#[macro_export]
macro_rules! tera_response_cache {
    ($cm:expr, $etag_if_none_match:expr, $key:expr, $gen:block) => {{
        let cm = &$cm;
        let etag_if_none_match: &$crate::EtagIfNoneMatch<'_> = &$etag_if_none_match;
        let key = &$key;
        let key = ::std::convert::AsRef::<str>::as_ref(key);

        match cm.build_from_cache(etag_if_none_match, key) {
            Some(res) => res,
            None => {
                let key = ::std::borrow::ToOwned::to_owned(key);
                // Copy the condition before the generator can take it.
                let etag_if_none_match = ::std::clone::Clone::clone(etag_if_none_match);
                let res = $gen;

                match res.into_cache_parts() {
                    Ok((content, etag)) => {
                        let res = if etag_if_none_match.weak_eq(&etag) {
                            $crate::TeraResponse::not_modified_with_etag(&etag)
                        } else {
                            $crate::TeraResponse::build_cache(content.clone(), etag.clone())
                        };

                        $cm.insert(key, (content, etag));

                        res
                    },
                    Err(res) => res,
                }
            },
        }
    }};
}
