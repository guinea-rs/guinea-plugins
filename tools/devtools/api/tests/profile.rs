//! A profile through the routes: seconds, the frames of one, and one frame
//! with what the application recorded towards it.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use guinea_devtools_api::Devtools;
use guinea_devtools_model::protocol::native::{Frame, Pass};
use guinea_devtools_model::protocol::{
    AppInfo, Capability, ClockAnchor, Report, Span, TraceBatch, TracePoint,
};
use guinea_devtools_model::sessions::{Incoming, Sessions};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

const QPC: u64 = 1_000_000;

fn report(id: u64, report: Report) -> Incoming {
    Incoming::Report(id, Box::new(report))
}

/// An application that sent a clock and a record, and its inspector with
/// two frames: one in time, one over the budget a second later.
fn sessions(inspected: bool) -> Sessions {
    let mut sessions = Sessions::default();
    sessions.apply(Incoming::Opened(1));
    sessions.apply(report(
        1,
        Report::Hello(AppInfo {
            name: "Processes".into(),
            pid: 42,
            clock: Some(ClockAnchor {
                qpc: QPC,
                qpc_frequency: 1_000_000,
                trace_us: 0,
                puffin_ns: None,
                ui_thread: 9,
            }),
            ..AppInfo::default()
        }),
    ));
    sessions.apply(report(
        1,
        Report::Trace(TraceBatch {
            spans: vec![Span {
                id: 7,
                parent: None,
                at: 1_250_000,
                took: None,
                point: TracePoint::Note {
                    text: "rows arrived".into(),
                },
            }],
            ends: Vec::new(),
            dropped: 0,
        }),
    ));

    if inspected {
        sessions.apply(Incoming::Opened(2));
        sessions.apply(report(
            2,
            Report::Hello(AppInfo {
                pid: 42,
                capabilities: vec![Capability::NativeTree],
                ..AppInfo::default()
            }),
        ));
        sessions.apply(report(
            2,
            Report::NativePerf {
                frames: vec![
                    Frame {
                        took_us: 4_000,
                        qpc: QPC + 200_000,
                        thread: 9,
                        ..Frame::default()
                    },
                    Frame {
                        took_us: 31_000,
                        measure_us: 20_000,
                        qpc: QPC + 1_240_000,
                        thread: 9,
                        passes: vec![Pass {
                            element: 5,
                            kind: "measure".into(),
                            took_us: 20_000,
                            at_us: 1_000,
                        }],
                        ..Frame::default()
                    },
                ],
            },
        ));
    }

    sessions
}

async fn ask(inspected: bool, url: &str) -> (StatusCode, Value) {
    let sessions = sessions(inspected);
    let devtools = Arc::new(Devtools::new(
        move |job| job(&sessions),
        |_, _| Err("nothing to send to".to_string()),
        |_| Err("no inspector here".to_string()),
    ));
    let (router, _) = guinea_devtools_api::api(devtools);

    let request = Request::builder()
        .uri(url)
        .body(Body::empty())
        .expect("a request");
    let response = router.oneshot(request).await.expect("an answer");
    let status = response.status();
    let body = response.into_body().collect().await.expect("a body").to_bytes();

    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn seconds_list_what_was_drawn_and_how_badly() {
    let (status, body) = ask(true, "/apps/1/seconds").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let seconds = body["seconds"].as_array().expect("seconds");
    assert_eq!(seconds.len(), 2, "{body}");
    assert_eq!(seconds[1]["t"], 1);
    assert_eq!(seconds[1]["at"], "1 s");
    assert_eq!(seconds[1]["over"], 1);
    assert_eq!(seconds[1]["worst_frame"], 1_240_000);
}

#[tokio::test]
async fn a_second_lists_its_frames_with_their_costliest_pass() {
    let (status, body) = ask(true, "/apps/1/seconds/1/frames").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let frames = body["frames"].as_array().expect("frames");
    assert_eq!(frames.len(), 1, "{body}");
    assert_eq!(frames[0]["frame"], 1_240_000);
    assert_eq!(frames[0]["over"], true);
    assert_eq!(frames[0]["costliest"]["kind"], "measure");
    assert_eq!(frames[0]["work"], 1, "one record towards it");
}

#[tokio::test]
async fn a_frame_says_what_the_application_recorded_towards_it() {
    let (status, body) = ask(true, "/apps/1/frames/1240000").await;
    assert_eq!(status, StatusCode::OK, "{body}");

    assert_eq!(body["took_us"], 31_000);
    assert_eq!(body["passes"][0]["element"], 5);
    let work = body["work"].as_array().expect("work");
    assert_eq!(work[0]["record"], 7);
    assert!(
        work[0]["says"].as_str().is_some_and(|says| says.contains("rows arrived")),
        "{body}"
    );

    let (status, _) = ask(true, "/apps/1/frames/123").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn without_an_inspector_there_is_nothing_to_profile() {
    let (status, body) = ask(false, "/apps/1/seconds").await;

    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert!(body.to_string().contains("attach"), "{body}");
}
