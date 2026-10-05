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

use std::cell::RefCell;
use std::time::Duration;

use guinea_devtools_api::tools::Tool;
use guinea_devtools_model::access::Access;
use serde_json::{Value, json};

/// What this server calls itself when a client asks.
const NAME: &str = "guinea-devtools";
/// The newest protocol this speaks; a client asking for an older one is
/// answered in its own.
const PROTOCOL: &str = "2025-06-18";
/// How long devtools have to accept a connection.
const CONNECT: Duration = Duration::from_secs(5);
/// How long a call may take: past the longest an action is followed, which is
/// a minute.
const ANSWER: Duration = Duration::from_secs(90);

fn main() -> anyhow::Result<()> {
    let devtools = Devtools::default();
    let tools = guinea_devtools_api::tools::tools(&guinea_devtools_api::document());

    stdio::serve(|request| rpc::answer(&devtools, &tools, request))
}

/// The devtools this talks to, found when first asked for and found again
/// when they stop answering - they pick a new token, and maybe a new port,
/// every time they start.
struct Devtools {
    found: RefCell<Option<Found>>,
    agent: ureq::Agent,
}

#[derive(Clone)]
struct Found {
    url: String,
    bearer: String,
}

/// What one try at a call came to.
enum Tried {
    Answered(Value),
    /// Refused as a stranger, or not answered at all: the devtools found may
    /// be gone, or started again since.
    Stale(Value),
}

impl Default for Devtools {
    fn default() -> Self {
        Self {
            found: RefCell::new(None),
            agent: ureq::Agent::new_with_config(
                ureq::Agent::config_builder()
                    .timeout_connect(Some(CONNECT))
                    .timeout_global(Some(ANSWER))
                    .http_status_as_error(false)
                    .build(),
            ),
        }
    }
}

impl Devtools {
    /// From the environment when it says, from the access file otherwise.
    fn find() -> Result<Found, String> {
        let url = std::env::var("GUINEA_DEVTOOLS_URL").ok();
        let token = std::env::var("GUINEA_DEVTOOLS_TOKEN").ok();

        let access = match (url, token) {
            (Some(url), Some(token)) => Access { url, token },
            _ => Access::read().map_err(|error| {
                format!(
                    "no devtools to talk to ({error}): start `guinea-devtools` or \
                     `guinea-devtools --headless`, or set GUINEA_DEVTOOLS_URL and GUINEA_DEVTOOLS_TOKEN"
                )
            })?,
        };

        Ok(Found {
            url: access.url.trim_end_matches('/').to_string(),
            bearer: format!("Bearer {}", access.token),
        })
    }

    fn found(&self) -> Result<Found, String> {
        if let Some(found) = self.found.borrow().clone() {
            return Ok(found);
        }

        let found = Self::find()?;
        *self.found.borrow_mut() = Some(found.clone());
        Ok(found)
    }

    /// Calls `tool` with `arguments`, and answers with what came back.
    fn call(&self, tool: &Tool, arguments: &serde_json::Map<String, Value>) -> Value {
        let path = match tool.url(arguments) {
            Ok(path) => path,
            Err(missing) => return said(&missing, true),
        };

        match self.try_call(tool, &path) {
            Tried::Answered(answer) => answer,
            Tried::Stale(_) => {
                self.found.borrow_mut().take();
                match self.try_call(tool, &path) {
                    Tried::Answered(answer) | Tried::Stale(answer) => answer,
                }
            }
        }
    }

    fn try_call(&self, tool: &Tool, path: &str) -> Tried {
        let found = match self.found() {
            Ok(found) => found,
            Err(why) => return Tried::Answered(said(&why, true)),
        };
        let url = format!("{}{path}", found.url);

        let bearer = found.bearer.as_str();
        let sent = match tool.method {
            "POST" => self
                .agent
                .post(&url)
                .header("Authorization", bearer)
                .send_empty(),
            "PUT" => self
                .agent
                .put(&url)
                .header("Authorization", bearer)
                .send_empty(),
            "DELETE" => self
                .agent
                .delete(&url)
                .header("Authorization", bearer)
                .call(),
            _ => self.agent.get(&url).header("Authorization", bearer).call(),
        };

        let mut response = match sent {
            Ok(response) => response,
            Err(error) => {
                return Tried::Stale(said(&format!("devtools are not answering: {error}"), true));
            }
        };
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string();

        if status < 400 {
            return Tried::Answered(match body {
                Ok(body) => said(&body, false),
                Err(error) => said(&format!("devtools answered unreadably: {error}"), true),
            });
        }

        let refused = said(
            &format!(
                "devtools refused with {status}: {}",
                body.unwrap_or_default()
            ),
            true,
        );
        if status == 401 {
            Tried::Stale(refused)
        } else {
            Tried::Answered(refused)
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
