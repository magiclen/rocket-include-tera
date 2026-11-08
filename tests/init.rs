use rocket::{
    State, get,
    http::{Header, Status},
    local::blocking::Client,
    routes,
};
use rocket_include_tera::{
    EtagIfNoneMatch, TeraContextManager, TeraResponse, tera_resources_initializer,
    tera_response_cache,
};
use serde_json::json;

#[get("/")]
fn index(cm: &State<TeraContextManager>, etag_if_none_match: EtagIfNoneMatch<'_>) -> TeraResponse {
    let data = json!({"title": "Title", "body": "Hello, world!"});

    rocket_include_tera::tera_response!(auto_minify cm, etag_if_none_match, "index", data)
}

#[get("/cached")]
fn cached_index(
    cm: &State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch<'_>,
) -> TeraResponse {
    tera_response_cache!(cm, etag_if_none_match, "index", { index(cm, EtagIfNoneMatch::default()) })
}

#[test]
fn fairing_cache() {
    let client = Client::untracked(
        rocket::build()
            .attach(tera_resources_initializer!(
                100;
                "index" => "examples/views/index.tera",
                "index2" => "examples/views/index2.tera"
            ))
            .mount("/", routes![index, cached_index]),
    )
    .unwrap();

    let fresh = client.get("/").dispatch();
    assert_eq!(Status::Ok, fresh.status());

    let etag = fresh.headers().get_one("ETag").unwrap().to_owned();

    for path in ["/", "/cached", "/cached"] {
        let response =
            client.get(path).header(Header::new("If-None-Match", etag.clone())).dispatch();

        assert_eq!(Status::NotModified, response.status());
        assert_eq!(Some(etag.as_str()), response.headers().get_one("ETag"));
        assert_eq!(None, response.into_string());

        #[cfg(not(debug_assertions))]
        if path == "/cached" {
            assert!(client.rocket().state::<TeraContextManager>().unwrap().contains_key("index"));
        }
    }

    let response = client.get("/cached").dispatch();
    assert_eq!(Status::Ok, response.status());
    assert_eq!(Some(etag.as_str()), response.headers().get_one("ETag"));
    assert!(response.into_string().unwrap().contains("Hello, world!"));
}

#[test]
fn fairing() {
    let client = Client::untracked(
        rocket::build()
            .attach(tera_resources_initializer!(
                "index" => "tests/views/child.tera",
                "base" => ("tests", "views", "base.tera")
            ))
            .mount("/", routes![index]),
    )
    .unwrap();

    let response = client.get("/").dispatch();
    assert_eq!(Status::Ok, response.status());
    assert!(response.into_string().unwrap().contains("Hello, world!"));
}
