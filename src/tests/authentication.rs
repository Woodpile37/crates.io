use crate::util::{MockRequestExt, RequestHelper, Response};
use crate::TestApp;

use crate::util::encode_session_header;
use http::{header, Method, StatusCode};

static URL: &str = "/api/v1/me/updates";
static MUST_LOGIN: &[u8] = br#"{"errors":[{"detail":"must be logged in to perform that action"}]}"#;

#[test]
fn anonymous_user_unauthorized() {
    let (_, anon) = TestApp::init().empty();
    let response: Response<()> = anon.get(URL);

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.json().to_string().as_bytes(), MUST_LOGIN);
}

#[test]
fn token_auth_cannot_find_token() {
    let (_, anon) = TestApp::init().empty();
    let mut request = anon.request_builder(Method::GET, URL);
    request.header(header::AUTHORIZATION, "cio1tkfake-token");
    let response: Response<()> = anon.run(request);

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.json().to_string().as_bytes(), MUST_LOGIN);
}

// Ensure that an unexpected authentication error is available for logging.  The user would see
// status 500 instead of 403 as in other authentication tests.  Due to foreign-key constraints in
// the database, it is not possible to implement this same test for a token.
#[test]
fn cookie_auth_cannot_find_user() {
    let (app, anon) = TestApp::init().empty();

    let session_key = app.as_inner().session_key();
    let cookie = encode_session_header(session_key, -1);

    let mut request = anon.request_builder(Method::GET, URL);
    request.header(header::COOKIE, &cookie);

    let error = anon.run::<()>(request);
    assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
