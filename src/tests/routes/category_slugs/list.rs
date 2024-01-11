use crate::new_category;
use crate::util::{RequestHelper, TestApp};
use insta::assert_json_snapshot;
use serde_json::Value;

#[test]
fn category_slugs_returns_all_slugs_in_alphabetical_order() {
    let (app, anon) = TestApp::init().empty();
    app.db(|conn| {
        new_category("Foo", "foo", "For crates that foo")
            .create_or_update(conn)
            .unwrap();
        new_category("Bar", "bar", "For crates that bar")
            .create_or_update(conn)
            .unwrap();
    });

    let response: Value = anon.get("/api/v1/category_slugs").good();
    assert_json_snapshot!(response);
}
