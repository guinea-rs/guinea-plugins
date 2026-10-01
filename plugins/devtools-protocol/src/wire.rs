//! A [`Report`](crate::Report) or a [`Command`](crate::Command) in and out of
//! a `Peer.send` call. Which one a side reads is fixed by which side it is.

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::devtools_capnp::peer;

pub fn encode(message: &impl Serialize) -> String {
    serde_json::to_string(message).expect("a message always serializes")
}

/// A message that does not parse is an error rather than skipped: both ends
/// are built from this crate, so a bad one means the versions disagree.
pub fn decode<T: DeserializeOwned>(json: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(json)
}

/// Sends `message` and waits for the other end to take it.
pub async fn send(peer: &peer::Client, message: &impl Serialize) -> Result<(), capnp::Error> {
    let mut request = peer.send_request();
    request.get().set_json(encode(message));
    request.send().promise.await.map(|_| ())
}

/// What arrived in a `send` call, read.
pub fn received<T: DeserializeOwned>(params: &peer::SendParams) -> Result<T, capnp::Error> {
    let json = params.get()?.get_json()?.to_str()?;
    decode(json).map_err(|error| capnp::Error::failed(format!("unreadable message: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Answer, AppInfo, BusKind, Capability, Command, End, Report, Span, TraceBatch, TracePoint,
    };

    #[test]
    fn commands_come_back_as_they_went() {
        let sent = [
            Command::NativeHitTest { x: -4, y: 900 },
            Command::NativeSetProperty {
                element: 7,
                property: 834,
                type_name: "Microsoft.UI.Xaml.HorizontalAlignment".into(),
                value: "Stretch".into(),
            },
            Command::NativeHighlight { element: None },
        ];

        for command in &sent {
            assert_eq!(&decode::<Command>(&encode(command)).unwrap(), command);
        }
    }

    #[test]
    fn native_reports_come_back_as_they_went() {
        use crate::native::{Bounds, Change, Element, Property};

        let sent = [
            Report::NativeTree {
                changes: vec![
                    Change::Added(Element {
                        handle: 2,
                        parent: 1,
                        index: 0,
                        kind: "Microsoft.UI.Xaml.Controls.Border".into(),
                        name: "Root".into(),
                        mark: "Processes".into(),
                    }),
                    Change::Removed {
                        handle: 3,
                        parent: 1,
                    },
                ],
            },
            Report::NativeProperties {
                element: 2,
                properties: vec![Property {
                    name: "HorizontalContentAlignment".into(),
                    value: "0".into(),
                    source: "built_in_style".into(),
                    index: 834,
                    ..Property::default()
                }],
            },
            Report::NativeEnums {
                enums: vec![crate::native::Enumeration {
                    name: "Microsoft.UI.Xaml.HorizontalAlignment".into(),
                    values: vec![(0, "Left".into()), (3, "Stretch".into())],
                }],
            },
            Report::NativePicked {
                chain: vec![2, 1],
                bounds: Some(Bounds {
                    x: -10,
                    y: 4,
                    width: 30,
                    height: 20,
                }),
            },
            Report::NativePerf {
                frames: vec![crate::native::Frame {
                    at_us: 16_000,
                    took_us: 2_400,
                    measure_us: 900,
                    arrange_us: 300,
                    passes: vec![crate::native::Pass {
                        element: 2,
                        kind: "measure".into(),
                        took_us: 700,
                    }],
                }],
            },
            Report::Refused {
                command: "NativeEdit".into(),
                reason: "no such property".into(),
            },
        ];

        for report in &sent {
            assert_eq!(&decode::<Report>(&encode(report)).unwrap(), report);
        }
    }

    #[test]
    fn a_capability_this_version_cannot_name_is_kept_as_unknown() {
        let json = r#"{"kind":"hello","name":"","identifier":"","version":"","backend":"","pid":1,"plugins":[],"capabilities":["snapshot","teleport"]}"#;

        let Report::Hello(info) = decode::<Report>(json).unwrap() else {
            panic!("a hello");
        };

        assert_eq!(
            info.capabilities,
            [Capability::Snapshot, Capability::Unknown]
        );
    }

    #[test]
    fn what_a_newer_peer_sends_and_this_one_cannot_name_reads_as_unknown() {
        let report = r#"{"kind":"teleported","to":"mars","by":3}"#;
        assert_eq!(decode::<Report>(report).unwrap(), Report::Unknown);

        let batch = r#"{"kind":"trace","spans":[
            {"id":1,"parent":null,"at":0,"took":null,"point":{"kind":"note","text":"before"}},
            {"id":2,"parent":1,"at":5,"took":null,"point":{"kind":"teleport","to":"mars"}},
            {"id":3,"parent":null,"at":9,"took":null,"point":{"kind":"note","text":"after"}}
        ],"ends":[],"dropped":0}"#;
        let Report::Trace(batch) = decode::<Report>(batch).unwrap() else {
            panic!("a batch");
        };
        let points: Vec<_> = batch.spans.iter().map(|span| span.point.kind()).collect();
        assert_eq!(
            points,
            ["note", "unknown", "note"],
            "the rest of the batch still reads"
        );

        let answered = r#"{"kind":"answered","request":7,"answer":{"kind":"beamed","at":1}}"#;
        assert_eq!(
            decode::<Report>(answered).unwrap(),
            Report::Answered {
                request: 7,
                answer: Answer::Unknown
            }
        );
    }

    #[test]
    fn a_hello_from_before_capabilities_lists_none() {
        let json = r#"{"kind":"hello","name":"","identifier":"","version":"","backend":"","pid":1,"plugins":[]}"#;

        let Report::Hello(info) = decode::<Report>(json).unwrap() else {
            panic!("a hello");
        };

        assert!(info.capabilities.is_empty());
    }

    #[test]
    fn reports_come_back_as_they_went() {
        let sent = [
            Report::Hello(AppInfo {
                name: "Processes".into(),
                pid: 7,
                ..AppInfo::default()
            }),
            Report::Trace(TraceBatch {
                spans: vec![Span {
                    id: 4,
                    parent: Some(3),
                    at: 1_500,
                    took: None,
                    point: TracePoint::Publish {
                        event: "multi\nline".into(),
                        bus: BusKind::Global,
                        subscribers: 2,
                    },
                }],
                ends: vec![End { id: 3, took: 20 }],
                dropped: 0,
            }),
        ];

        for report in &sent {
            assert_eq!(&decode::<Report>(&encode(report)).unwrap(), report);
        }
    }
}
