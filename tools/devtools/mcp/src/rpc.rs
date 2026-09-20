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

/// What to write back, if anything.
pub fn answer(devtools: &Devtools, tools: &[Tool], request: Request) -> Option<Value> {
    let id = request.id.clone()?;

    let answered = match request.method.as_str() {
        "initialize" => Ok(initialize(&request.params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": listed(tools) })),
        "tools/call" => call(devtools, tools, &request.params),
        other => Err(format!("no such method: {other}")),
    };

    Some(match answered {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(message) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": message },
        }),
    })
}

fn initialize(params: &Value) -> Value {
    let asked = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL);

    json!({
        "protocolVersion": asked,
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

fn call(devtools: &Devtools, tools: &[Tool], params: &Value) -> Result<Value, String> {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or("a call says which tool")?;

    let tool = tools
        .iter()
        .find(|tool| tool.name == name)
        .ok_or_else(|| format!("no such tool: {name}"))?;

    let empty = Map::new();
    let arguments = params
        .get("arguments")
        .and_then(Value::as_object)
        .unwrap_or(&empty);

    Ok(devtools.call(tool, arguments))
}
