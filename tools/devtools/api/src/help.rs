//! The document as a person reads it: what `--help` prints.
//!
//! Rendered from the same OpenAPI the API serves, so a route that exists is
//! a route that is documented, and one that is gone stops being mentioned.

use utoipa::openapi::path::{Operation, Parameter, ParameterIn};
use utoipa::openapi::{HttpMethod, OpenApi, RefOr};

/// How wide the whole thing is allowed to be.
const WIDTH: usize = 80;
/// Where a summary starts, after the method and the path.
const COLUMN: usize = 46;

/// Every route, by tag, with what it answers and what it takes.
pub fn routes(document: &OpenApi) -> String {
    let mut out = String::new();

    if let Some(about) = document.info.description.as_deref() {
        out.push_str(&wrap(about, 0));
        out.push('\n');
    }

    for tag in tags(document) {
        let about = document
            .tags
            .iter()
            .flatten()
            .find(|listed| listed.name == tag)
            .and_then(|listed| listed.description.clone())
            .unwrap_or_default();

        out.push('\n');
        out.push_str(&format!("{tag} - {about}\n"));

        for (path, method, operation) in operations(document) {
            if !operation.tags.iter().flatten().any(|listed| *listed == tag) {
                continue;
            }

            out.push_str(&line(&path, method, operation));
        }
    }

    out
}

/// The tags in the order the document lists them, then any a route names
/// that the document does not.
fn tags(document: &OpenApi) -> Vec<String> {
    let mut tags: Vec<String> = document
        .tags
        .iter()
        .flatten()
        .map(|tag| tag.name.clone())
        .collect();

    for (_, _, operation) in operations(document) {
        for tag in operation.tags.iter().flatten() {
            if !tags.contains(tag) {
                tags.push(tag.clone());
            }
        }
    }

    tags
}

fn operations(document: &OpenApi) -> Vec<(String, HttpMethod, &Operation)> {
    document
        .paths
        .paths
        .iter()
        .flat_map(|(path, item)| {
            [
                (HttpMethod::Get, &item.get),
                (HttpMethod::Post, &item.post),
                (HttpMethod::Put, &item.put),
                (HttpMethod::Patch, &item.patch),
                (HttpMethod::Delete, &item.delete),
            ]
            .into_iter()
            .filter_map(move |(method, operation)| {
                operation
                    .as_ref()
                    .map(|operation| (path.clone(), method, operation))
            })
        })
        .collect()
}

/// The parameters an operation spells out; the handlers declare every one
/// in place, so none is a reference.
fn parameters(operation: &Operation) -> impl Iterator<Item = &Parameter> {
    operation
        .parameters
        .iter()
        .flatten()
        .filter_map(|parameter| match parameter {
            RefOr::T(parameter) => Some(parameter),
            RefOr::Ref(_) => None,
        })
}

fn line(path: &str, method: HttpMethod, operation: &Operation) -> String {
    let verb = match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
        HttpMethod::Head => "HEAD",
        HttpMethod::Options => "OPTIONS",
        HttpMethod::Trace => "TRACE",
    };

    let left = format!("  {verb:<5}{path}");
    let summary = operation.summary.clone().unwrap_or_default();

    let mut out = if summary.is_empty() {
        format!("{left}\n")
    } else if left.len() >= COLUMN {
        format!("{left}\n{}\n", wrap(&summary, COLUMN))
    } else {
        let padding = " ".repeat(COLUMN - left.len());
        let wrapped = wrap(&summary, COLUMN);

        format!("{left}{padding}{}\n", wrapped.trim_start())
    };

    for query in parameters(operation) {
        if !matches!(query.parameter_in, ParameterIn::Query) {
            continue;
        }

        let about = query.description.clone().unwrap_or_default();
        let name = format!("        ?{}=", query.name);

        out.push_str(&if about.is_empty() {
            format!("{name}\n")
        } else if name.len() >= COLUMN {
            format!("{name}\n{}\n", wrap(&about, COLUMN))
        } else {
            let padding = " ".repeat(COLUMN - name.len());

            format!("{name}{padding}{}\n", wrap(&about, COLUMN).trim_start())
        });
    }

    out
}

/// `text` as lines that fit, every line but the first indented by `at`.
fn wrap(text: &str, at: usize) -> String {
    let width = WIDTH.saturating_sub(at).max(20);
    let indent = " ".repeat(at);

    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }

    lines.join(&format!("\n{indent}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_route_says_what_it_answers_and_where_it_belongs() {
        let document = crate::document();

        for (path, _, operation) in operations(&document) {
            assert!(
                operation.summary.is_some(),
                "{path} has no summary, so --help would print a bare line"
            );
            assert!(
                operation.tags.iter().flatten().next().is_some(),
                "{path} has no tag, so --help would not know where to put it"
            );
        }
    }

    /// utoipa reads where a parameter lives off the extractor's name, so an
    /// import renamed to dodge a clash silently turns a query into a path
    /// parameter - and then `--help`, and anything else reading the document,
    /// stops mentioning it.
    #[test]
    fn a_parameter_is_in_the_path_only_when_the_path_names_it() {
        for (path, _, operation) in operations(&crate::document()) {
            assert!(
                operation
                    .parameters
                    .iter()
                    .flatten()
                    .all(|parameter| matches!(parameter, RefOr::T(_))),
                "{path} refers to a parameter rather than declaring it, so --help would skip it"
            );
            for parameter in parameters(operation) {
                let named = path.contains(&format!("{{{}}}", parameter.name));
                let in_path = matches!(parameter.parameter_in, ParameterIn::Path);

                assert_eq!(
                    in_path, named,
                    "{path} says {} is a path parameter: {in_path}, and the path says {named}",
                    parameter.name
                );
            }
        }
    }

    #[test]
    fn help_prints_every_route_under_a_heading() {
        let document = crate::document();
        let printed = routes(&document);

        for (path, _, _) in operations(&document) {
            assert!(printed.contains(&path), "{path} is missing from --help");
        }
        assert!(printed.contains("apps - "), "the tags are headings");

        println!("{printed}");
    }
}
