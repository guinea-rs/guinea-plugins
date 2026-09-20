//! The routes answering, without a port and without an application: the hub
//! is a closure over whatever the test hands them.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use guinea_devtools_api::Devtools;
use guinea_devtools_model::protocol::{AppInfo, Report};
use guinea_devtools_model::sessions::{Incoming, Sessions};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Sessions with one connected application in them.
fn sessions() -> Sessions {
    let mut sessions = Sessions::default();
    sessions.apply(Incoming::Opened(1));
    sessions.apply(Incoming::Report(
        1,
        Box::new(Report::Hello(AppInfo {
            name: "Processes".into(),
            identifier: "dev.uniproc.guinea.processes".into(),
            pid: 42,
            ..AppInfo::default()
        })),
    ));

    sessions
}

fn devtools() -> Arc<Devtools> {
    let sessions = sessions();

    Arc::new(Devtools::new(
        move |job| job(&sessions),
        |_, _| Err("nothing to send to".to_string()),
        |_| Err("no inspector here".to_string()),
    ))
}

/// The status and the body of what the routes answer.
async fn ask(method: &str, url: &str) -> (StatusCode, String) {
    let (router, _) = guinea_devtools_api::api(devtools());

    let request = Request::builder()
        .method(method)
        .uri(url)
        .body(Body::empty())
        .expect("a request");

    let response = router.oneshot(request).await.expect("an answer");
    let status = response.status();
    let body = response.into_body().collect().await.expect("a body").to_bytes();

    (status, String::from_utf8_lossy(&body).into_owned())
}

#[tokio::test]
async fn the_newest_application_answers_to_latest() {
    let (status, body) = ask("GET", "/apps").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("Processes"), "{body}");

    let (status, body) = ask("GET", "/apps/latest/snapshot").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = ask("GET", "/apps/1/elements").await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn an_application_nobody_knows_is_a_404() {
    let (status, body) = ask("GET", "/apps/9000/snapshot").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("no application 9000"), "{body}");
}

#[tokio::test]
async fn a_query_that_does_not_parse_is_a_400() {
    let (status, _) = ask("GET", "/apps/latest/element?id=not-an-element").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_command_with_no_inspector_to_send_it_to_is_a_404() {
    let (status, body) = ask("POST", "/apps/latest/native/perf").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body.contains("inspector"), "{body}");
}

#[tokio::test]
async fn reading_a_route_that_wants_a_post_says_so() {
    let (status, _) = ask("GET", "/apps/latest/native/perf").await;

    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
}
