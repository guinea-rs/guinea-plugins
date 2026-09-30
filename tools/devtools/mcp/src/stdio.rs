//! The transport: JSON-RPC, one message per line, in and out.

use std::io::{BufRead, Write};

use serde_json::Value;

use crate::rpc::{Refusal, Request, refused};

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

        let answered = match read(&line) {
            Read::Request(request) => answer(request),
            Read::Response => None,
            Read::Unparsed(why) => Some(refused(Value::Null, Refusal(Refusal::UNPARSED, why))),
        };

        if let Some(answer) = answered {
            writeln!(output, "{answer}")?;
            output.flush()?;
        }
    }

    Ok(())
}

/// What a line turned out to be.
enum Read {
    Request(Request),
    /// A client's answer to something this never asks; nothing to say to it.
    Response,
    Unparsed(String),
}

fn read(line: &str) -> Read {
    let message: Value = match serde_json::from_str(line) {
        Ok(message) => message,
        Err(error) => return Read::Unparsed(format!("not JSON: {error}")),
    };
    let Some(method) = message.get("method") else {
        return Read::Response;
    };
    let Some(method) = method.as_str() else {
        return Read::Unparsed("a method is a string".into());
    };

    Read::Request(Request {
        id: message.get("id").cloned(),
        method: method.to_string(),
        params: message.get("params").cloned().unwrap_or(Value::Null),
    })
}
