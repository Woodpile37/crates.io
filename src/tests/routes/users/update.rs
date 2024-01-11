use crate::util::{RequestHelper, Response, TestApp};
use http::StatusCode;

pub trait MockEmailHelper: RequestHelper {
    // TODO: I don't like the name of this method or `update_email` on the `MockCookieUser` impl;
    // this is starting to look like a builder might help?
    // I want to explore alternative abstractions in any case.
    fn update_email_more_control(&self, user_id: i32, email: Option<&str>) -> Response<()> {
        // When updating your email in crates.io, the request goes to the user route with PUT.
        // Ember sends all the user attributes. We check to make sure the ID in the URL matches
        // the ID of the currently logged in user, then we ignore everything but the email address.
        let body = json!({"user": {
            "email": email,
            "name": "Arbitrary Name",
            "login": "arbitrary_login",
            "avatar": "https://arbitrary.com/img.jpg",
            "url": "https://arbitrary.com",
            "kind": null
        }});
        let url = format!("/api/v1/users/{user_id}");
        self.put(&url, body.to_string())
    }
}

impl MockEmailHelper for crate::util::MockCookieUser {}
impl MockEmailHelper for crate::util::MockAnonymousUser {}

impl crate::util::MockCookieUser {
    pub fn update_email(&self, email: &str) {
        let model = self.as_model();
        let response = self.update_email_more_control(model.id, Some(email));
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.json(), json!({ "ok": true }));
    }
}

/// Given a crates.io user, check to make sure that the user
/// cannot add to the database an empty string or null as
/// their email. If an attempt is made, update_user.rs will
/// return an error indicating that an empty email cannot be
/// added.
///
/// This is checked on the frontend already, but I'd like to
/// make sure that a user cannot get around that and delete
/// their email by adding an empty string.
#[test]
fn test_empty_email_not_added() {
    let (_app, _anon, user) = TestApp::init().with_user();
    let model = user.as_model();

    let response = user.update_email_more_control(model.id, Some(""));
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "empty email rejected" }] })
    );

    let response = user.update_email_more_control(model.id, None);
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "empty email rejected" }] })
    );
}

/// Check to make sure that neither other signed in users nor anonymous users can edit another
/// user's email address.
///
/// If an attempt is made, update_user.rs will return an error indicating that the current user
/// does not match the requested user.
#[test]
fn test_other_users_cannot_change_my_email() {
    let (app, anon, user) = TestApp::init().with_user();
    let another_user = app.db_new_user("not_me");
    let another_user_model = another_user.as_model();

    let response = user.update_email_more_control(
        another_user_model.id,
        Some("pineapple@pineapples.pineapple"),
    );
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "current user does not match requested user" }] })
    );

    let response = anon.update_email_more_control(
        another_user_model.id,
        Some("pineapple@pineapples.pineapple"),
    );
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(
        response.json(),
        json!({ "errors": [{ "detail": "must be logged in to perform that action" }] })
    );
}
