use crate::builders::PublishBuilder;
use crate::util::{RequestHelper, TestApp};
use googletest::prelude::*;
use http::StatusCode;
use insta::assert_json_snapshot;

#[test]
fn good_keywords() {
    let (_, _, _, token) = TestApp::full().with_token();
    let crate_to_publish = PublishBuilder::new("foo_good_key", "1.0.0")
        .keyword("c++")
        .keyword("crates-io_index")
        .keyword("1password");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json(), {
        ".crate.created_at" => "[datetime]",
        ".crate.updated_at" => "[datetime]",
    });
}

#[test]
fn bad_keywords() {
    let (_, _, _, token) = TestApp::full().with_token();
    let crate_to_publish =
        PublishBuilder::new("foo_bad_key", "1.0.0").keyword("super-long-keyword-name-oh-no");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());

    let crate_to_publish = PublishBuilder::new("foo_bad_key", "1.0.0").keyword("?@?%");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());

    let crate_to_publish = PublishBuilder::new("foo_bad_key", "1.0.0").keyword("áccênts");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());
}

#[test]
fn too_many_keywords() {
    let (app, _, _, token) = TestApp::full().with_token();
    let response = token.publish_crate(
        PublishBuilder::new("foo", "1.0.0")
            .keyword("one")
            .keyword("two")
            .keyword("three")
            .keyword("four")
            .keyword("five")
            .keyword("six"),
    );
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());
    assert_that!(app.stored_files(), empty());
}
