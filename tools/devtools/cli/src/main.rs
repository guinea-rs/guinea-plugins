//! guinea devtools in a terminal.
//!
//! Asks a running devtools, window or `--headless`, over their HTTP API, and
//! prints what the model made of it. Every command takes `--json` for the
//! answer as it came.

use std::time::Duration;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};
use guinea_devtools_model::access::Access;
use guinea_devtools_model::chains::{StreamLine, StreamView};
use guinea_devtools_model::elements::{Details, Line};
use guinea_devtools_model::graph::{EdgeKind, Graph};
use guinea_devtools_model::panels::Listed;
use guinea_devtools_model::protocol::Node;
use guinea_devtools_model::sessions::Summary;
use guinea_devtools_model::trace::{Cause, Consequence, Record, Row};
use guinea_devtools_model::words::text;
use serde::de::DeserializeOwned;

/// guinea devtools in a terminal.
///
/// Devtools must be running: `guinea-devtools` or `guinea-devtools --headless`.
#[derive(Parser)]
#[command(version)]
struct Cli {
    /// Which application: its id, or `latest`.
    #[arg(long, global = true, default_value = "latest")]
    app: String,

    /// Print the answer as it came, in JSON.
    #[arg(long, global = true)]
    json: bool,

    /// Where devtools answer, instead of what their access file says.
    #[arg(long, global = true, env = "GUINEA_DEVTOOLS_URL", requires = "token")]
    url: Option<String>,

    /// The token that goes with `--url`.
    #[arg(long, global = true, env = "GUINEA_DEVTOOLS_TOKEN", hide_env_values = true)]
    token: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Applications that connected.
    Apps,

    /// The element tree; ids in brackets.
    Elements {
        /// Elements whose children are left out, comma-separated.
        #[arg(long, default_value = "")]
        closed: String,
    },

    /// One element.
    Element {
        /// `app`, `actor/7`, `segment/1/0`, `feature/1/0/TabsFeature`, ...
        id: String,
    },

    /// The last records.
    Trace {
        /// Kinds left out, comma-separated: action send handle spawn publish
        /// deliver push navigate store log tick note.
        #[arg(long, default_value = "tick")]
        hide: String,

        /// Only records that say this.
        #[arg(long, default_value = "")]
        text: String,

        /// How many of the newest records.
        #[arg(long, default_value_t = 50)]
        limit: usize,

        /// Keep printing records as they come.
        #[arg(long)]
        follow: bool,
    },

    /// One record: where it came from, what it set off.
    Record {
        id: u64,

        /// Kinds left out of what happened in between.
        #[arg(long, default_value = "tick")]
        hide: String,
    },

    /// What started the chains of records: actions, timers, loops.
    Streams,

    /// One stream's chains, those of one shape as one line.
    Chains {
        /// `all`, `action/c::Kill`, `timer/<place>`, `loop/<step>`, ...; as
        /// `streams --json` gives them.
        #[arg(default_value = "all")]
        stream: String,

        /// Only chains that say this.
        #[arg(long, default_value = "")]
        text: String,
    },

    /// What plugins contributed.
    Panels,

    /// Actors, reducers, buses and the edges between.
    Graph,

    /// The last snapshot, as the application sent it.
    Snapshot,

    /// Where devtools answer.
    Url,
}

struct Client {
    access: Access,
}

impl Client {
    fn new(url: Option<String>, token: Option<String>) -> anyhow::Result<Self> {
        let access = match (url, token) {
            (Some(url), Some(token)) => Access { url, token },
            _ => Access::read().context("devtools are not running, or never were: no access file")?,
        };

        Ok(Self { access })
    }

    fn text(&self, path: &str, query: &[(&str, String)]) -> anyhow::Result<String> {
        let mut request = ureq::get(&format!("{}{path}", self.access.url))
            .set("Authorization", &format!("Bearer {}", self.access.token));

        for (name, value) in query {
            request = request.query(name, value);
        }

        match request.call() {
            Ok(response) => Ok(response.into_string()?),
            Err(ureq::Error::Status(status, response)) => {
                let body = response.into_string().unwrap_or_default();
                bail!("devtools said {status}: {body}")
            }
            Err(error) => Err(error).context("devtools do not answer; are they running?"),
        }
    }

    fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, String)]) -> anyhow::Result<T> {
        Ok(serde_json::from_str(&self.text(path, query)?)?)
    }

    /// Prints the answer at `path`: as it came with `json`, otherwise with
    /// `print`.
    fn answer<T: DeserializeOwned>(
        &self,
        json: bool,
        path: &str,
        query: &[(&str, String)],
        print: impl FnOnce(T),
    ) -> anyhow::Result<()> {
        if json {
            println!("{}", self.text(path, query)?);
        } else {
            print(self.get(path, query)?);
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = Client::new(cli.url, cli.token)?;
    let json = cli.json;
    let base = format!("/apps/{}", cli.app);

    match cli.command {
        Command::Url => println!("{}", client.access.url),
        Command::Apps => client.answer(json, "/apps", &[], print_apps)?,
        Command::Elements { closed } => client.answer(
            json,
            &format!("{base}/elements"),
            &[("closed", closed)],
            print_elements,
        )?,
        Command::Element { id } => {
            client.answer(json, &format!("{base}/element"), &[("id", id)], print_details)?
        }
        Command::Trace {
            hide,
            text,
            limit,
            follow: false,
        } => client.answer(
            json,
            &format!("{base}/trace"),
            &[("hide", hide), ("text", text), ("limit", limit.to_string())],
            |rows: Vec<Row>| print_rows(&rows),
        )?,
        Command::Trace {
            hide,
            text,
            limit,
            follow: true,
        } => follow(&client, json, &format!("{base}/trace"), hide, text, limit)?,
        Command::Record { id, hide } => client.answer(
            json,
            &format!("{base}/trace/{id}"),
            &[("hide", hide)],
            print_record,
        )?,
        Command::Streams => client.answer(json, &format!("{base}/streams"), &[], print_streams)?,
        Command::Chains { stream, text } => client.answer(
            json,
            &format!("{base}/chains"),
            &[("stream", stream), ("text", text)],
            print_chains,
        )?,
        Command::Panels => client.answer(json, &format!("{base}/panels"), &[], print_panels)?,
        Command::Graph => client.answer(json, &format!("{base}/graph"), &[], print_graph)?,
        Command::Snapshot => println!("{}", client.text(&format!("{base}/snapshot"), &[])?),
    }

    Ok(())
}

/// The last `limit` records, then every new one as it comes.
fn follow(
    client: &Client,
    json: bool,
    path: &str,
    hide: String,
    text: String,
    limit: usize,
) -> anyhow::Result<()> {
    let mut after = 0;
    let mut limit = limit;

    loop {
        let query = [
            ("hide", hide.clone()),
            ("text", text.clone()),
            ("limit", limit.to_string()),
            ("after", after.to_string()),
        ];
        let rows: Vec<Row> = client.get(path, &query)?;

        if let Some(last) = rows.last() {
            after = last.id;
        }

        if json {
            for row in &rows {
                println!("{}", serde_json::to_string(row)?);
            }
        } else {
            print_rows(&rows);
        }

        limit = 1000;
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn print_apps(apps: Vec<Summary>) {
    if apps.is_empty() {
        println!("no application has connected");
    }

    for app in apps {
        println!(
            "{:>3}  {}  {} · pid {} · {}  {}  {} records",
            app.id,
            app.name,
            app.backend,
            app.pid,
            app.version,
            if app.connected { "connected" } else { "gone" },
            app.records,
        );
    }
}

fn print_elements(lines: Vec<Line>) {
    for line in lines {
        let marker = match (line.branch, line.open) {
            (false, _) => "  ",
            (true, true) => "▾ ",
            (true, false) => "▸ ",
        };

        println!(
            "{}{marker}{}  [{}]",
            "  ".repeat(line.depth),
            text(&line.words),
            line.element
        );
    }
}

fn print_details(details: Details) {
    println!("{} ({})", details.title, details.kind);
    if let Some(declared) = &details.declared {
        println!("declared at {}:{}:{}", declared.file, declared.line, declared.column);
    }

    let width = details.rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    for (name, value) in &details.rows {
        println!("  {name:>width$}  {value}");
    }

    if !details.handlers.is_empty() {
        println!("handlers:");
    }
    for handler in &details.handlers {
        let flows = if handler.flows.is_empty() {
            String::new()
        } else {
            format!(" → {}", handler.flows)
        };

        let place = handler
            .declared
            .as_ref()
            .map(|declared| format!("  at {}:{}:{}", declared.file, declared.line, declared.column))
            .unwrap_or_default();

        println!("  {}{flows}{place}", handler.message);
    }

    if let Some(body) = &details.body {
        println!();
        println!("{body}");
    }
}

fn cause(row: &Row) -> String {
    match &row.cause {
        Some(Cause::Known { id, gist, .. }) => format!("  ← {gist} #{id}"),
        Some(Cause::Forgotten) => "  ← (forgotten)".to_string(),
        None => String::new(),
    }
}

fn line(row: &Row) -> String {
    format!(
        "#{:<6} {}  {}{}{}",
        row.id,
        row.time,
        text(&row.words),
        row.took.as_ref().map(|took| format!("  {took}")).unwrap_or_default(),
        cause(row),
    )
}

fn print_rows(rows: &[Row]) {
    for row in rows {
        println!("{}", line(row));
    }
}

fn print_consequences(set_off: &[Consequence], depth: usize) {
    for consequence in set_off {
        println!("{}{}", "  ".repeat(depth + 1), line(&consequence.row));
        print_consequences(&consequence.children, depth + 1);
    }
}

fn print_record(record: Record) {
    println!("{}", line(&record.row));
    println!("  {}", record.row.when);

    println!();
    println!("where it came from:");
    if let Some(note) = &record.origins.note {
        println!("  {note}");
    }

    for (depth, step) in record.origins.steps.iter().enumerate() {
        println!("{}{}", "  ".repeat(depth + 1), line(&step.row));

        for between in &step.between {
            println!("{}· {}", "  ".repeat(depth + 2), line(between));
        }

        if step.more > 0 {
            println!("{}· … and {} more in between", "  ".repeat(depth + 2), step.more);
        }
    }

    println!();
    println!("what it set off:");
    if record.set_off.is_empty() {
        println!("  nothing observed");
    }

    print_consequences(&record.set_off, 0);
    if record.cut {
        println!("  … more than devtools walk at once");
    }
}

fn print_nodes(nodes: &[Node], depth: usize) {
    for node in nodes {
        println!("{}{}  {}", "  ".repeat(depth), node.label, node.kind);
        for (name, value) in &node.properties {
            println!("{}  {name}: {value}", "  ".repeat(depth + 1));
        }
        print_nodes(&node.children, depth + 1);
    }
}

fn print_streams(streams: Vec<StreamLine>) {
    if streams.is_empty() {
        println!("nothing started a chain yet");
    }

    let mut section = None;
    for line in streams {
        if section != Some(line.section) {
            section = Some(line.section);
            println!("{:?}", line.section);
        }

        println!("  {:>6}  {}  [{}]", line.chains, line.title, line.stream);
    }
}

fn print_chains(view: StreamView) {
    println!("{}  [{}]", view.title, view.stream);

    if let Some(timer) = &view.timer {
        let place = timer
            .declared
            .as_ref()
            .map(|declared| format!("{}:{}", declared.file, declared.line))
            .unwrap_or_default();
        println!(
            "  every {} · {} ticks · average {} · longest {} · {} running  {place}",
            timer.period.as_deref().unwrap_or("?"),
            timer.fires,
            timer.average.as_deref().unwrap_or("–"),
            timer.longest.as_deref().unwrap_or("–"),
            timer.running,
        );
    }

    println!();
    for group in &view.groups {
        let longest = group
            .longest
            .as_ref()
            .map(|longest| format!("  up to {longest}"))
            .unwrap_or_default();
        let opens = group
            .opens
            .as_ref()
            .map(|stream| format!("  [{stream}]"))
            .unwrap_or_default();
        println!(
            "{}  ×{:<5} {}{longest}{opens}",
            group.time,
            group.count,
            text(&group.words)
        );

        for run in &group.runs {
            println!("         #{:<8} {}  {}", run.root, run.time, run.took.as_deref().unwrap_or("–"));
        }
    }
}

fn print_panels(panels: Vec<Listed>) {
    if panels.is_empty() {
        println!("nothing contributed a panel");
    }

    for listed in panels {
        println!("{}  [{}]", listed.title, listed.key);
        print_nodes(&listed.panel.nodes, 1);
    }
}

fn print_graph(graph: Graph) {
    for (at, cluster) in graph.clusters.iter().enumerate() {
        let depth = std::iter::successors(cluster.parent, |parent| graph.clusters[*parent].parent).count();
        println!("{}{} ({:?})", "  ".repeat(depth), cluster.label, cluster.kind);

        for node in graph.nodes.iter().filter(|node| node.cluster == at) {
            println!("{}- {} ({:?})", "  ".repeat(depth + 1), node.label, node.kind);
        }
    }

    println!();
    for edge in &graph.edges {
        let how = match edge.kind {
            EdgeKind::Flow(channel) => format!("{channel:?}").to_lowercase(),
            other => format!("{other:?}").to_lowercase(),
        };

        println!(
            "{} -{how}-> {}{}{}",
            graph.nodes[edge.from].label,
            graph.nodes[edge.to].label,
            if edge.label.is_empty() { String::new() } else { format!("  {}", edge.label) },
            if edge.recent > 0 { format!("  ×{} recently", edge.recent) } else { String::new() },
        );
    }
}
