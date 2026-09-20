//! guinea devtools as MCP tools: one tool per route, over stdio.
//!
//! The tools are not written here. They are the operations of the devtools
//! OpenAPI document - name, arguments and all - so a route that appears in
//! the API appears here, and one that changes its arguments changes them
//! here too.
//!
//! State stays where it is: this proxies to a running devtools, window or
//! `--headless`, found through the access file they write at startup.

mod rpc;
mod stdio;

use anyhow::Context;
use guinea_devtools_api::tools::Tool;
use guinea_devtools_model::access::Access;
use serde_json::{Value, json};

/// What this server calls itself when a client asks.
const NAME: &str = "guinea-devtools";
/// The newest protocol this speaks; a client asking for an older one is
/// answered in its own.
const PROTOCOL: &str = "2025-06-18";

fn main() -> anyhow::Result<()> {
    let devtools = Devtools::found()?;
    let tools = guinea_devtools_api::tools::tools(&guinea_devtools_api::document());

    stdio::serve(|request| rpc::answer(&devtools, &tools, request))
}

/// The devtools this talks to.
struct Devtools {
    url: String,
    bearer: String,
}

impl Devtools {
    /// From the environment when it says, from the access file otherwise.
    fn found() -> anyhow::Result<Devtools> {
        let url = std::env::var("GUINEA_DEVTOOLS_URL").ok();
        let token = std::env::var("GUINEA_DEVTOOLS_TOKEN").ok();

        let access = match (url, token) {
            (Some(url), Some(token)) => Access { url, token },
            _ => Access::read().context(
                "no devtools to talk to: start `guinea-devtools` or `guinea-devtools --headless`, \
                 or set GUINEA_DEVTOOLS_URL and GUINEA_DEVTOOLS_TOKEN",
            )?,
        };

        Ok(Devtools {
            url: access.url.trim_end_matches('/').to_string(),
            bearer: format!("Bearer {}", access.token),
        })
    }

    /// Calls `tool` with `arguments`, and answers with what came back.
    fn call(&self, tool: &Tool, arguments: &serde_json::Map<String, Value>) -> Value {
        let url = match tool.url(arguments) {
            Ok(url) => format!("{}{url}", self.url),
            Err(missing) => return said(&missing, true),
        };

        let request = match tool.method {
            "POST" => ureq::post(&url),
            "PUT" => ureq::put(&url),
            "DELETE" => ureq::delete(&url),
            _ => ureq::get(&url),
        };

        match request.set("Authorization", &self.bearer).call() {
            Ok(response) => match response.into_string() {
                Ok(body) => said(&body, false),
                Err(error) => said(&format!("devtools answered unreadably: {error}"), true),
            },
            Err(ureq::Error::Status(status, response)) => {
                let body = response.into_string().unwrap_or_default();

                said(&format!("devtools refused with {status}: {body}"), true)
            }
            Err(error) => said(&format!("devtools are not answering: {error}"), true),
        }
    }
}

/// One text answer, as a tool result.
fn said(text: &str, failed: bool) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": failed,
    })
}
