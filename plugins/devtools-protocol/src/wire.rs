//! A [`Report`] in and out of a `Peer.send` call.

use crate::Report;
use crate::devtools_capnp::peer;

pub fn encode(report: &Report) -> String {
    serde_json::to_string(report).expect("a report always serializes")
}

/// A message that does not parse is an error rather than skipped: both ends
/// are built from this crate, so a bad one means the versions disagree.
pub fn decode(json: &str) -> Result<Report, serde_json::Error> {
    serde_json::from_str(json)
}

/// Sends `report` and waits for the other end to take it.
pub async fn send(peer: &peer::Client, report: &Report) -> Result<(), capnp::Error> {
    let mut request = peer.send_request();
    request.get().set_json(encode(report));
    request.send().promise.await.map(|_| ())
}

/// What arrived in a `send` call, read.
pub fn received(params: &peer::SendParams) -> Result<Report, capnp::Error> {
    let json = params.get()?.get_json()?.to_str()?;
    decode(json).map_err(|error| capnp::Error::failed(format!("unreadable report: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppInfo, BusKind, End, Span, TraceBatch, TracePoint};

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
            assert_eq!(&decode(&encode(report)).unwrap(), report);
        }
    }
}
