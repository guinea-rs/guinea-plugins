//! The transport: JSON-RPC, one message per line, in and out.

use std::io::{BufRead, Write};

use serde_json::Value;

use crate::rpc::Request;

/// Reads requests until the client goes away, writing back what `answer`
/// returns.
pub fn serve(mut answer: impl FnMut(Request) -> Option<Value>) -> anyhow::Result<()> {
    let input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();

    for line in input.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let Some(request) = read(&line) else {
            continue;
        };

        if let Some(answer) = answer(request) {
            writeln!(output, "{answer}")?;
            output.flush()?;
        }
    }

    Ok(())
}

/// A line as a request, or nothing when it is not one: a client that sends
/// rubbish is not worth stopping for.
fn read(line: &str) -> Option<Request> {
    let message: Value = serde_json::from_str(line).ok()?;
    let method = message.get("method")?.as_str()?.to_string();

    Some(Request {
        id: message.get("id").cloned(),
        method,
        params: message.get("params").cloned().unwrap_or(Value::Null),
    })
}
