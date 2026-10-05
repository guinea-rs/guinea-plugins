//! What the client may ask, and what it is told.

use guinea_devtools_api::tools::Tool;
use serde_json::{Map, Value, json};

use crate::{Devtools, NAME, PROTOCOL};

/// A request as it arrived: an id when it wants an answer, none when it is a
/// notification.
pub struct Request {
    pub id: Option<Value>,
    pub method: String,
    pub params: Value,
}

/// The protocol versions this can speak, newest first.
const SPOKEN: [&str; 3] = [PROTOCOL, "2025-03-26", "2024-11-05"];

/// A JSON-RPC error: its code, and what it says.
pub struct Refusal(pub i64, pub String);

impl Refusal {
    pub const UNPARSED: i64 = -32700;
    const NO_METHOD: i64 = -32601;
    const BAD_PARAMS: i64 = -32602;
}

/// What to write back, if anything.
pub fn answer(devtools: &Devtools, tools: &[Tool], request: Request) -> Option<Value> {
    let id = request.id.clone()?;

    let answered = match request.method.as_str() {
        "initialize" => Ok(initialize(&request.params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": listed(tools) })),
        "tools/call" => call(devtools, tools, &request.params),
        other => Err(Refusal(
            Refusal::NO_METHOD,
            format!("no such method: {other}"),
        )),
    };

    Some(match answered {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(refusal) => refused(id, refusal),
    })
}

/// `refusal`, as the answer to request `id`.
pub fn refused(id: Value, Refusal(code, message): Refusal) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn initialize(params: &Value) -> Value {
    let asked = params.get("protocolVersion").and_then(Value::as_str);
    let spoken = SPOKEN
        .into_iter()
        .find(|version| Some(*version) == asked)
        .unwrap_or(PROTOCOL);

    json!({
        "protocolVersion": spoken,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": { "name": NAME, "version": env!("CARGO_PKG_VERSION") },
        "instructions": "Looks inside a running guinea application: what it is made of, \
    what it did, and what its backend draws. `app` is an application's id as `list_apps` \
    gives it, or `latest` for the newest one still connected.",
    })
}

fn listed(tools: &[Tool]) -> Vec<Value> {
    tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.about,
                "inputSchema": tool.schema,
            })
        })
        .collect()
}

fn call(devtools: &Devtools, tools: &[Tool], params: &Value) -> Result<Value, Refusal> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| Refusal(Refusal::BAD_PARAMS, "a call says which tool".into()))?;

    let tool = tools
        .iter()
        .find(|tool| tool.name == name)
        .ok_or_else(|| Refusal(Refusal::BAD_PARAMS, format!("no such tool: {name}")))?;

    let empty = Map::new();
    if params
        .get("arguments")
        .is_some_and(|arguments| !arguments.is_object())
    {
        return Err(Refusal(
            Refusal::BAD_PARAMS,
            "arguments are an object".into(),
        ));
    }
    let arguments = params
        .get("arguments")
        .and_then(Value::as_object)
        .unwrap_or(&empty);

    Ok(devtools.call(tool, arguments))
}
