use crate::builders::{CrateBuilder, PublishBuilder};
use crate::util::{RequestHelper, TestApp};
use googletest::prelude::*;
use http::StatusCode;
use insta::assert_json_snapshot;

#[test]
fn new_crate_similar_name() {
    let (app, _, user, token) = TestApp::full().with_token();

    app.db(|conn| {
        CrateBuilder::new("Foo_similar", user.as_model().id)
            .version("1.0.0")
            .expect_build(conn);
    });

    let crate_to_publish = PublishBuilder::new("foo_similar", "1.1.0");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());

    assert_that!(app.stored_files(), empty());
}

#[test]
fn new_crate_similar_name_hyphen() {
    let (app, _, user, token) = TestApp::full().with_token();

    app.db(|conn| {
        CrateBuilder::new("foo_bar_hyphen", user.as_model().id)
            .version("1.0.0")
            .expect_build(conn);
    });

    let crate_to_publish = PublishBuilder::new("foo-bar-hyphen", "1.1.0");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());

    assert_that!(app.stored_files(), empty());
}

#[test]
fn new_crate_similar_name_underscore() {
    let (app, _, user, token) = TestApp::full().with_token();

    app.db(|conn| {
        CrateBuilder::new("foo-bar-underscore", user.as_model().id)
            .version("1.0.0")
            .expect_build(conn);
    });

    let crate_to_publish = PublishBuilder::new("foo_bar_underscore", "1.1.0");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());

    assert_that!(app.stored_files(), empty());
}
