//! The document as something to call: one tool per operation, with the
//! schema of its arguments and how to turn them into a request.
//!
//! What reads this is the MCP server, but nothing here knows that: a tool is
//! a name, what it is for, and the URL it becomes.

use serde_json::{Map, Value, json};
use utoipa::openapi::OpenApi;

/// One operation, ready to be offered and called.
#[derive(Clone, Debug, PartialEq)]
pub struct Tool {
    /// The operation's id, which is what a caller names.
    pub name: String,
    /// What it answers, and what it takes.
    pub about: String,
    pub method: &'static str,
    /// The path with its `{placeholders}` still in it.
    pub path: String,
    /// JSON Schema of the arguments, as an object.
    pub schema: Value,
    parameters: Vec<Parameter>,
}

#[derive(Clone, Debug, PartialEq)]
struct Parameter {
    name: String,
    in_path: bool,
    required: bool,
}

impl Tool {
    /// The path and query string for these arguments, or what is missing.
    pub fn url(&self, arguments: &Map<String, Value>) -> Result<String, String> {
        let mut path = self.path.clone();
        let mut query = form_urlencoded::Serializer::new(String::new());

        for parameter in &self.parameters {
            let Some(value) = arguments.get(&parameter.name) else {
                if parameter.required {
                    return Err(format!("{} wants {}", self.name, parameter.name));
                }
                continue;
            };

            let value = match value {
                Value::String(text) => text.clone(),
                Value::Null => continue,
                other => other.to_string(),
            };

            if parameter.in_path {
                path = path.replace(&format!("{{{}}}", parameter.name), &segment(&value));
            } else {
                query.append_pair(&parameter.name, &value);
            }
        }

        let query = query.finish();

        Ok(if query.is_empty() {
            path
        } else {
            format!("{path}?{query}")
        })
    }
}

/// `value` as one path segment: everything but the unreserved characters
/// percent-encoded, so a `/`, `?` or `#` in it cannot lead somewhere else.
fn segment(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                (byte as char).to_string()
            }
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Every operation the document describes, in the order it lists them.
pub fn tools(document: &OpenApi) -> Vec<Tool> {
    let described = serde_json::to_value(document).unwrap_or_else(|_| json!({}));
    let paths = described.get("paths").and_then(Value::as_object);

    let mut tools = Vec::new();

    for (path, item) in paths.into_iter().flatten() {
        for method in ["get", "post", "put", "patch", "delete"] {
            let Some(operation) = item.get(method).and_then(Value::as_object) else {
                continue;
            };

            tools.push(tool(path, method, operation));
        }
    }

    tools
}

fn tool(path: &str, method: &str, operation: &Map<String, Value>) -> Tool {
    let text = |key: &str| {
        operation
            .get(key)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };

    let name = match operation.get("operationId").and_then(Value::as_str) {
        Some(id) => id.to_string(),
        None => format!("{method}{}", path.replace(['/', '{', '}'], "_")),
    };

    let mut about = text("summary");
    let description = text("description");
    if !description.is_empty() && description != about {
        about = format!("{about}\n\n{description}");
    }

    let declared: Vec<&Map<String, Value>> = operation
        .get("parameters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_object)
        .collect();

    let mut properties = Map::new();
    let mut required = Vec::new();
    let mut parameters = Vec::new();

    for parameter in declared {
        let name = parameter
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if name.is_empty() {
            continue;
        }

        let mut schema = parameter
            .get("schema")
            .cloned()
            .unwrap_or_else(|| json!({ "type": "string" }));

        if let (Some(schema), Some(about)) = (schema.as_object_mut(), parameter.get("description"))
        {
            schema.entry("description").or_insert_with(|| about.clone());
        }

        let is_required = parameter
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if is_required {
            required.push(Value::String(name.clone()));
        }

        parameters.push(Parameter {
            name: name.clone(),
            in_path: parameter.get("in").and_then(Value::as_str) == Some("path"),
            required: is_required,
        });
        properties.insert(name, schema);
    }

    Tool {
        name,
        about,
        method: match method {
            "post" => "POST",
            "put" => "PUT",
            "patch" => "PATCH",
            "delete" => "DELETE",
            _ => "GET",
        },
        path: path.to_string(),
        schema: json!({
            "type": "object",
            "properties": Value::Object(properties),
            "required": Value::Array(required),
        }),
        parameters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> Tool {
        tools(&crate::document())
            .into_iter()
            .find(|tool| tool.name == name)
            .unwrap_or_else(|| panic!("no tool {name}"))
    }

    #[test]
    fn every_operation_becomes_a_tool_with_a_name_of_its_own() {
        let tools = tools(&crate::document());
        assert!(!tools.is_empty(), "the document describes something");

        let mut names: Vec<&str> = tools.iter().map(|tool| tool.name.as_str()).collect();
        names.sort_unstable();
        let taken = names.len();
        names.dedup();

        assert_eq!(names.len(), taken, "two operations share a name: {names:?}");

        for tool in &tools {
            assert!(
                !tool.about.is_empty(),
                "{} says nothing about itself",
                tool.name
            );
        }
    }

    #[test]
    fn arguments_become_a_path_and_a_query() {
        let trace = named("get_trace");
        let arguments = json!({ "app": "latest", "hide": "tick,render", "limit": 20 });

        assert_eq!(
            trace
                .url(arguments.as_object().expect("an object"))
                .expect("a url"),
            "/apps/latest/trace?hide=tick%2Crender&limit=20"
        );
    }

    #[test]
    fn a_path_argument_stays_in_its_segment() {
        let trace = named("get_trace");
        let arguments = json!({ "app": "1/../2?x#y" });

        assert_eq!(
            trace
                .url(arguments.as_object().expect("an object"))
                .expect("a url"),
            "/apps/1%2F..%2F2%3Fx%23y/trace"
        );
    }

    #[test]
    fn what_a_tool_needs_it_says_it_needs() {
        let element = named("get_element");
        let missing = element.url(&Map::new()).expect_err("id is wanted");

        assert!(
            missing.contains("app") || missing.contains("id"),
            "{missing}"
        );
    }
}
