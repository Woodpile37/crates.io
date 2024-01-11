use crate::builders::{CrateBuilder, PublishBuilder};
use crate::util::{RequestHelper, TestApp};
use crates_io::schema::api_tokens;
use diesel::{ExpressionMethods, RunQueryDsl};
use googletest::prelude::*;
use http::StatusCode;
use insta::assert_json_snapshot;

#[test]
fn new_wrong_token() {
    let (app, anon, _, token) = TestApp::full().with_token();

    // Try to publish without a token
    let crate_to_publish = PublishBuilder::new("foo", "1.0.0");
    let response = anon.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "must be logged in to perform that action" }] })
    );

    // Try to publish with the wrong token (by changing the token in the database)
    app.db(|conn| {
        diesel::update(api_tokens::table)
            .set(api_tokens::token.eq(b"bad" as &[u8]))
            .execute(conn)
            .unwrap();
    });

    let crate_to_publish = PublishBuilder::new("foo", "1.0.0");
    let response = token.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "must be logged in to perform that action" }] })
    );
    assert_that!(app.stored_files(), empty());
}

#[test]
fn new_krate_wrong_user() {
    let (app, _, user) = TestApp::full().with_user();

    app.db(|conn| {
        // Create the foo_wrong crate with one user
        CrateBuilder::new("foo_wrong", user.as_model().id).expect_build(conn);
    });

    // Then try to publish with a different user
    let another_user = app.db_new_user("another").db_new_token("bar");
    let crate_to_publish = PublishBuilder::new("foo_wrong", "2.0.0");

    let response = another_user.publish_crate(crate_to_publish);
    assert_eq!(response.status(), StatusCode::OK);
    assert_json_snapshot!(response.json());

    assert_that!(app.stored_files(), empty());
}
