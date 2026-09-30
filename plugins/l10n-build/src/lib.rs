use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use fluent_syntax::parser::parse;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use syn::Path as SynPath;

#[derive(Debug, Clone, PartialEq)]
pub struct L10nMessage {
    pub id: String,
    pub variables: Vec<String>,
    /// The `.ftl` it was written in, relative to the locale's own directory:
    /// `main.ftl`, `menu/file.ftl`. Empty when the file stands alone.
    pub file: String,
    /// The line the message starts on, counting from one.
    pub line: u32,
    /// What the reference locale says, as written - placeables and all.
    pub text: String,
}

pub fn parse_messages(ftl_path: &Path) -> Vec<L10nMessage> {
    parse_messages_of(ftl_path, "")
}

/// [`parse_messages`], naming the file as devtools should show it.
fn parse_messages_of(ftl_path: &Path, file: &str) -> Vec<L10nMessage> {
    let content = fs::read_to_string(ftl_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", ftl_path.display()));
    let resource = parse(content.as_str())
        .unwrap_or_else(|(_, errors)| panic!("failed to parse {}: {errors:?}", ftl_path.display()));

    let mut messages: Vec<L10nMessage> = resource
        .body
        .into_iter()
        .filter_map(|entry| match entry {
            Entry::Message(msg) => {
                let id = msg.id.name.to_string();
                let mut variables = Vec::new();
                if let Some(pattern) = &msg.value {
                    collect_variables(pattern, &mut variables);
                }
                let line = line_of(&content, &id);
                Some(L10nMessage {
                    text: text_at(&content, line),
                    line,
                    id,
                    variables,
                    file: file.to_string(),
                })
            }
            _ => None,
        })
        .collect();
    messages.sort_by(|a, b| a.id.cmp(&b.id));
    messages
}

/// The message written at `line`: what follows the `=`, and the indented
/// lines under it, joined by a space.
fn text_at(content: &str, line: u32) -> String {
    if line == 0 {
        return String::new();
    }

    let mut lines = content.lines().skip(line as usize - 1);
    let Some((_, first)) = lines.next().and_then(|first| first.split_once('=')) else {
        return String::new();
    };

    let mut text = first.trim().to_string();
    for next in lines {
        if next.trim().is_empty() || !next.starts_with([' ', '\t']) {
            break;
        }
        text.push(' ');
        text.push_str(next.trim());
    }

    text.trim().to_string()
}

/// The line `id` is defined on. The parser drops spans, and a message starts
/// its own line, so the line is the one that begins with the identifier.
fn line_of(content: &str, id: &str) -> u32 {
    content
        .lines()
        .position(|line| {
            line.strip_prefix(id)
                .is_some_and(|rest| rest.trim_start().starts_with('='))
        })
        .map_or(0, |at| at as u32 + 1)
}

fn collect_variables(pattern: &Pattern<&str>, out: &mut Vec<String>) {
    for element in &pattern.elements {
        if let PatternElement::Placeable { expression } = element {
            collect_from_expression(expression, out);
        }
    }
}

fn collect_from_expression(expr: &Expression<&str>, out: &mut Vec<String>) {
    match expr {
        Expression::Inline(inline) => collect_from_inline(inline, out),
        Expression::Select { selector, variants } => {
            collect_from_inline(selector, out);
            for variant in variants {
                collect_variables(&variant.value, out);
            }
        }
    }
}

fn collect_from_inline(inline: &InlineExpression<&str>, out: &mut Vec<String>) {
    match inline {
        InlineExpression::VariableReference { id } => push_unique(out, id.name),
        InlineExpression::FunctionReference { arguments, .. } => collect_call_args(arguments, out),
        InlineExpression::TermReference { arguments, .. } => {
            if let Some(arguments) = arguments {
                collect_call_args(arguments, out);
            }
        }
        InlineExpression::Placeable { expression } => collect_from_expression(expression, out),
        InlineExpression::StringLiteral { .. }
        | InlineExpression::NumberLiteral { .. }
        | InlineExpression::MessageReference { .. } => {}
    }
}

fn collect_call_args(arguments: &fluent_syntax::ast::CallArguments<&str>, out: &mut Vec<String>) {
    for positional in &arguments.positional {
        collect_from_inline(positional, out);
    }
    for named in &arguments.named {
        collect_from_inline(&named.value, out);
    }
}

fn push_unique(out: &mut Vec<String>, name: &str) {
    if !out.iter().any(|v| v == name) {
        out.push(name.to_string());
    }
}

/// Rust's keywords, strict and reserved, through edition 2024: a name among
/// them is written raw.
const KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do", "dyn",
    "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl", "in", "let",
    "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return",
    "static", "struct", "trait", "true", "try", "type", "typeof", "unsafe", "unsized", "use",
    "virtual", "where", "while", "yield",
];

/// Keywords that cannot be written raw either.
const UNRAWABLE: &[&str] = &["self", "Self", "super", "crate"];

/// What `fluent_loader!` already defines on the resolver, and the
/// `Localization` methods an accessor would shadow.
const RESOLVER_METHODS: &[&str] = &[
    "new",
    "language",
    "current",
    "get_raw",
    "for_tag",
    "tag",
    "keys",
    "languages",
    "value",
];

/// What an accessor's own body calls its arguments.
const ACCESSOR_LOCALS: &[&str] = &["args"];

/// `name` as a Rust identifier: `-` and `.` become `_`, a keyword is written
/// raw, and a name that is taken, or cannot be raw, gets a `_` after it.
fn ident_for(name: &str, taken: &[&str]) -> Ident {
    let snake = name.replace(['-', '.'], "_");

    if taken.contains(&snake.as_str()) || UNRAWABLE.contains(&snake.as_str()) {
        format_ident!("{snake}_")
    } else if KEYWORDS.contains(&snake.as_str()) {
        Ident::new_raw(&snake, Span::call_site())
    } else {
        Ident::new(&snake, Span::call_site())
    }
}

/// Where `message` is written, for an error to point at.
fn written_at(message: &L10nMessage) -> String {
    let file = if message.file.is_empty() {
        "the reference locale"
    } else {
        message.file.as_str()
    };
    format!("{file}:{}", message.line)
}

/// Panics, naming both, when two messages come out as one method or two
/// variables of a message as one argument.
fn refuse_collisions(messages: &[L10nMessage]) {
    let mut methods: HashMap<String, &L10nMessage> = HashMap::new();

    for message in messages {
        let method = ident_for(&message.id, RESOLVER_METHODS).to_string();
        if let Some(first) = methods.insert(method.clone(), message) {
            panic!(
                "l10n: `{}` ({}) and `{}` ({}) would both be the method `{method}`; rename one",
                first.id,
                written_at(first),
                message.id,
                written_at(message),
            );
        }

        let mut arguments: HashMap<String, &str> = HashMap::new();
        for variable in &message.variables {
            let argument = ident_for(variable, ACCESSOR_LOCALS).to_string();
            if let Some(first) = arguments.insert(argument.clone(), variable) {
                panic!(
                    "l10n: `{}` ({}) has `${first}` and `${variable}`, which would both be the argument `{argument}`; rename one",
                    message.id,
                    written_at(message),
                );
            }
        }
    }
}

pub fn generate_l10n_accessors(
    messages: &[L10nMessage],
    resolver_path: &str,
    fluent_args_path: &str,
    fluent_value_path: &str,
) -> TokenStream {
    let resolver_path: SynPath = syn::parse_str(resolver_path)
        .unwrap_or_else(|e| panic!("invalid resolver_path {resolver_path:?}: {e}"));
    let fluent_args_path: SynPath = syn::parse_str(fluent_args_path)
        .unwrap_or_else(|e| panic!("invalid fluent_args_path {fluent_args_path:?}: {e}"));
    let fluent_value_path: SynPath = syn::parse_str(fluent_value_path)
        .unwrap_or_else(|e| panic!("invalid fluent_value_path {fluent_value_path:?}: {e}"));

    refuse_collisions(messages);

    let methods = messages.iter().map(|msg| {
        let method_name = ident_for(&msg.id, RESOLVER_METHODS);
        let id = &msg.id;

        let params = msg.variables.iter().map(|v| {
            let param = ident_for(v, ACCESSOR_LOCALS);
            quote! { #param: impl Into<#fluent_value_path<'static>> }
        });

        if msg.variables.is_empty() {
            quote! {
                pub fn #method_name(&self) -> String {
                    self.get_raw(#id, &#fluent_args_path::new())
                }
            }
        } else {
            let inserts = msg.variables.iter().map(|v| {
                let param = ident_for(v, ACCESSOR_LOCALS);
                quote! { args.set(#v, #param.into()); }
            });
            quote! {
                pub fn #method_name(&self, #(#params),*) -> String {
                    let mut args = #fluent_args_path::new();
                    #(#inserts)*
                    self.get_raw(#id, &args)
                }
            }
        }
    });

    quote! {
        #[allow(dead_code)]
        impl #resolver_path {
            #(#methods)*
        }
    }
}

/// Every `.ftl` of one locale, in either layout: `<tag>.ftl` beside its
/// siblings, or `<tag>/**/*.ftl`. Empty when the locale has none.
fn locale_files(locales_dir: &Path, tag: &str) -> Vec<std::path::PathBuf> {
    let flat = locales_dir.join(format!("{tag}.ftl"));
    if flat.is_file() {
        return vec![flat];
    }

    let nested = locales_dir.join(tag);
    if !nested.is_dir() {
        return Vec::new();
    }

    let mut files: Vec<_> = walkdir::WalkDir::new(&nested)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "ftl"))
        .map(|e| e.into_path())
        .collect();
    files.sort();
    files
}

fn discover_locale_files(locales_dir: &Path, reference_locale: &str) -> Vec<std::path::PathBuf> {
    let files = locale_files(locales_dir, reference_locale);
    assert!(
        !files.is_empty(),
        "no `.ftl` files found for locale {reference_locale:?} under {} - expected either {} or {}/**/*.ftl",
        locales_dir.display(),
        locales_dir
            .join(format!("{reference_locale}.ftl"))
            .display(),
        locales_dir.join(reference_locale).display(),
    );

    files
}

pub fn parse_locale_messages(locales_dir: &Path, reference_locale: &str) -> Vec<L10nMessage> {
    // In the nested layout a file is named by its path under the locale's
    // directory, so `menu/file.ftl` stays apart from `file.ftl`. In the flat
    // one the locale *is* the file, and there is nothing to nest.
    let nested = locales_dir.join(reference_locale);
    let named = |path: &Path| match path.strip_prefix(&nested) {
        Ok(under) => under.to_string_lossy().replace('\\', "/"),
        Err(_) => String::new(),
    };

    let mut messages: Vec<L10nMessage> = discover_locale_files(locales_dir, reference_locale)
        .iter()
        .flat_map(|path| parse_messages_of(path, &named(path)))
        .collect();
    messages.sort_by(|a, b| a.id.cmp(&b.id));
    messages
}

/// Every locale beside `reference_locale`, as its directory or file names it.
pub fn locales(locales_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(locales_dir) else {
        return Vec::new();
    };

    let mut tags: Vec<String> = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                return path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned());
            }
            if path.extension().is_some_and(|extension| extension == "ftl") {
                return path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().into_owned());
            }
            None
        })
        .collect();

    tags.sort();
    tags
}

/// What every other locale says for each of `messages`, as written, by
/// reading every locale the way the reference one was read. A locale that
/// left a message out is not in its list.
fn translations_by_id(
    locales_dir: &Path,
    reference_locale: &str,
    messages: &[L10nMessage],
) -> Vec<Vec<(String, String)>> {
    let others: Vec<(String, Vec<L10nMessage>)> = locales(locales_dir)
        .into_iter()
        .filter(|tag| tag != reference_locale)
        .map(|tag| {
            let written = locale_files(locales_dir, &tag)
                .iter()
                .flat_map(|path| parse_messages(path))
                .collect();
            (tag, written)
        })
        .collect();

    messages
        .iter()
        .map(|message| {
            others
                .iter()
                .filter_map(|(tag, written)| {
                    written
                        .iter()
                        .find(|other| other.id == message.id)
                        .map(|other| (tag.clone(), other.text.clone()))
                })
                .collect()
        })
        .collect()
}

/// The table devtools read: every message, where it is written, what it takes,
/// what each locale says for it and which have not translated it yet; and
/// every locale there is.
pub fn generate_l10n_keys(
    messages: &[L10nMessage],
    translations: &[Vec<(String, String)>],
    reference_locale: &str,
    languages: &[String],
    keys_path: &str,
) -> TokenStream {
    let keys_path: SynPath = syn::parse_str(keys_path)
        .unwrap_or_else(|e| panic!("invalid keys_path {keys_path:?}: {e}"));

    let entries = messages
        .iter()
        .zip(translations)
        .map(|(message, translated)| {
            let id = &message.id;
            let file = &message.file;
            let line = message.line;
            let text = &message.text;
            let variables = &message.variables;
            let tags = translated.iter().map(|(tag, _)| tag.as_str());
            let texts = translated.iter().map(|(_, text)| text.as_str());
            let missing = languages
                .iter()
                .filter(|tag| *tag != reference_locale)
                .filter(|tag| !translated.iter().any(|(done, _)| done == *tag))
                .map(String::as_str);

            quote! {
                #keys_path {
                    id: #id,
                    file: #file,
                    line: #line,
                    text: #text,
                    variables: &[#(#variables),*],
                    translations: &[#((#tags, #texts)),*],
                    missing: &[#(#missing),*],
                }
            }
        });

    quote! {
        /// Every message of the reference locale, as `l10n` compiled it.
        pub static L10N_KEYS: &[#keys_path] = &[#(#entries),*];
        /// Every locale the application has, the reference one among them.
        pub static L10N_LANGUAGES: &[&str] = &[#(#languages),*];
    }
}

pub fn write_keys(locales_dir: &Path, reference_locale: &str, out_path: &Path) {
    let messages = parse_locale_messages(locales_dir, reference_locale);
    let translations = translations_by_id(locales_dir, reference_locale, &messages);
    let languages = locales(locales_dir);
    let generated = generate_l10n_keys(
        &messages,
        &translations,
        reference_locale,
        &languages,
        "guinea_plugin_l10n::Key",
    );

    fs::write(out_path, generated.to_string())
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}

pub fn write_accessors(locales_dir: &Path, reference_locale: &str, out_path: &Path) {
    let messages = parse_locale_messages(locales_dir, reference_locale);
    let generated = generate_l10n_accessors(
        &messages,
        "crate::l10n::L10n",
        "crate::l10n::Args",
        "fluent_bundle::FluentValue",
    );
    fs::write(out_path, generated.to_string())
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}

pub fn build(locales_dir: impl AsRef<Path>) {
    build_for_locale(locales_dir, "en")
}

pub fn build_for_locale(locales_dir: impl AsRef<Path>, reference_locale: &str) {
    let locales_dir = locales_dir.as_ref();
    println!("cargo:rerun-if-changed={}", locales_dir.display());

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set");
    write_accessors(
        locales_dir,
        reference_locale,
        &Path::new(&out_dir).join("l10n_accessors.rs"),
    );
    write_keys(
        locales_dir,
        reference_locale,
        &Path::new(&out_dir).join("l10n_keys.rs"),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_ftl(content: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp .ftl file");
        file.write_all(content.as_bytes())
            .expect("failed to write .ftl content");
        file
    }

    fn write_file(dir: &Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    /// A message as a test writes one: what it is called and what it takes,
    /// with where it was written filled in.
    fn message(id: &str, variables: &[&str], line: u32, text: &str) -> L10nMessage {
        L10nMessage {
            id: id.into(),
            variables: variables.iter().map(|name| name.to_string()).collect(),
            file: String::new(),
            line,
            text: text.into(),
        }
    }

    #[test]
    fn plain_message_has_no_variables() {
        let file = write_ftl("hello-world = Hello, World!\n");
        let messages = parse_messages(file.path());
        assert_eq!(
            messages,
            vec![message("hello-world", &[], 1, "Hello, World!")]
        );
    }

    #[test]
    fn placeable_variable_is_collected() {
        let file = write_ftl("welcome = Welcome, { $userName }.\n");
        let messages = parse_messages(file.path());
        assert_eq!(
            messages,
            vec![message(
                "welcome",
                &["userName"],
                1,
                "Welcome, { $userName }."
            )]
        );
    }

    #[test]
    fn a_message_knows_the_line_and_the_text_it_was_written_as() {
        let file = write_ftl("first = One\n\nsecond =\n    Over two\n    lines\n");
        let messages = parse_messages(file.path());

        let second = messages
            .iter()
            .find(|message| message.id == "second")
            .expect("second");
        assert_eq!(second.line, 3);
        assert_eq!(second.text, "Over two lines");
    }

    #[test]
    fn selector_and_variant_variables_are_all_collected_once_each() {
        let file = write_ftl(
            "emails = { $count ->\n    [one] { $count } email\n   *[other] { $count } emails, from { $sender }\n}\n",
        );
        let messages = parse_messages(file.path());
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].id, "emails");
        assert_eq!(
            messages[0].variables,
            vec!["count".to_string(), "sender".to_string()]
        );
    }

    #[test]
    fn non_message_entries_are_skipped() {
        let file = write_ftl("-brand-name = Nightly\n## a comment\nreal-message = Value\n");
        let messages = parse_messages(file.path());
        assert_eq!(messages, vec![message("real-message", &[], 3, "Value")]);
    }

    #[test]
    fn generates_zero_arg_method_for_plain_message() {
        let messages = vec![message("hello-world", &[], 1, "Hello, World!")];
        let generated = generate_l10n_accessors(
            &messages,
            "crate::l10n::L10n",
            "fluent_bundle::FluentArgs",
            "fluent_bundle::FluentValue",
        )
        .to_string();

        assert!(generated.contains("fn hello_world (& self) -> String"));
        assert!(generated.contains("fluent_bundle :: FluentArgs :: new"));
        assert!(generated.contains("get_raw (\"hello-world\""));
    }

    #[test]
    fn generates_one_param_per_variable_in_first_seen_order() {
        let messages = vec![message(
            "emails",
            &["count", "sender"],
            1,
            "{ $count } from { $sender }",
        )];
        let generated = generate_l10n_accessors(
            &messages,
            "crate::l10n::L10n",
            "fluent_bundle::FluentArgs",
            "fluent_bundle::FluentValue",
        )
        .to_string();

        assert!(generated.contains("fn emails"), "{generated}");
        assert!(generated.contains("count : impl Into"), "{generated}");
        assert!(generated.contains("sender : impl Into"), "{generated}");
        assert!(generated.contains("args . set (\"count\""), "{generated}");
        assert!(generated.contains("args . set (\"sender\""), "{generated}");
        assert!(
            generated.find("count").unwrap() < generated.find("sender").unwrap(),
            "params must appear in first-seen order: {generated}"
        );
    }

    fn accessors(messages: &[L10nMessage]) -> String {
        generate_l10n_accessors(
            messages,
            "crate::l10n::L10n",
            "fluent_bundle::FluentArgs",
            "fluent_bundle::FluentValue",
        )
        .to_string()
    }

    #[test]
    fn names_rust_will_not_take_are_raw_or_moved_aside() {
        let generated = accessors(&[
            message("continue", &[], 1, "Continue"),
            message("language", &[], 2, "Language"),
            message("self", &[], 3, "Me"),
            message("greeting", &["user-name", "type", "args"], 4, "Hi"),
        ]);

        assert!(generated.contains("fn r#continue (& self)"), "{generated}");
        assert!(generated.contains("fn language_ (& self)"), "{generated}");
        assert!(generated.contains("fn self_ (& self)"), "{generated}");
        assert!(generated.contains("user_name : impl Into"), "{generated}");
        assert!(generated.contains("r#type : impl Into"), "{generated}");
        assert!(generated.contains("args_ : impl Into"), "{generated}");
        assert!(
            generated.contains("args . set (\"user-name\" , user_name . into ())"),
            "{generated}"
        );
        assert!(
            generated.contains("args . set (\"args\" , args_ . into ())"),
            "{generated}"
        );
        syn::parse_file(&generated).expect("the accessors parse as Rust");
    }

    #[test]
    #[should_panic(
        expected = "`sign-in` (main.ftl:1) and `sign_in` (main.ftl:2) would both be the method `sign_in`"
    )]
    fn two_messages_that_would_be_one_method_are_refused() {
        let mut first = message("sign-in", &[], 1, "Sign in");
        let mut second = message("sign_in", &[], 2, "Sign in");
        first.file = "main.ftl".into();
        second.file = "main.ftl".into();
        accessors(&[first, second]);
    }

    #[test]
    #[should_panic(expected = "`$user-name` and `$user_name`")]
    fn two_variables_that_would_be_one_argument_are_refused() {
        accessors(&[message("hi", &["user-name", "user_name"], 1, "Hi")]);
    }

    #[test]
    fn flat_layout_finds_locale_dot_ftl() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "en.ftl", "hello-world = Hello, World!\n");
        write_file(dir.path(), "ru.ftl", "hello-world = Привет, мир!\n");

        let messages = parse_locale_messages(dir.path(), "en");
        assert_eq!(
            messages,
            vec![message("hello-world", &[], 1, "Hello, World!")]
        );
    }

    #[test]
    fn nested_layout_merges_every_ftl_file_under_the_locale_dir() {
        let dir = tempfile::tempdir().unwrap();
        write_file(dir.path(), "en/main.ftl", "hello-world = Hello, World!\n");
        write_file(dir.path(), "en/extra.ftl", "goodbye = Goodbye!\n");
        write_file(dir.path(), "ru/main.ftl", "hello-world = Привет, мир!\n");

        let messages = parse_locale_messages(dir.path(), "en");
        let named: Vec<(&str, &str)> = messages
            .iter()
            .map(|message| (message.id.as_str(), message.file.as_str()))
            .collect();

        assert_eq!(
            named,
            vec![("goodbye", "extra.ftl"), ("hello-world", "main.ftl")]
        );
    }

    #[test]
    fn a_locale_that_left_a_message_out_is_named_as_missing_it() {
        let dir = tempfile::tempdir().unwrap();
        write_file(
            dir.path(),
            "en/main.ftl",
            "kept = Kept\nleft-out = Left out\n",
        );
        write_file(dir.path(), "ru/main.ftl", "kept = Оставлено\n");

        let messages = parse_locale_messages(dir.path(), "en");
        let translations = translations_by_id(dir.path(), "en", &messages);
        let languages = locales(dir.path());

        assert_eq!(languages, vec!["en".to_string(), "ru".to_string()]);
        assert_eq!(
            translations,
            vec![
                vec![("ru".to_string(), "Оставлено".to_string())],
                Vec::new()
            ]
        );

        let generated =
            generate_l10n_keys(&messages, &translations, "en", &languages, "crate::Key")
                .to_string();
        assert!(
            generated.contains("translations : & [(\"ru\" , \"Оставлено\")]"),
            "{generated}"
        );
        assert!(generated.contains("missing : & [\"ru\"]"), "{generated}");
        assert!(
            generated.contains("L10N_LANGUAGES : & [& str] = & [\"en\" , \"ru\"]"),
            "{generated}"
        );
    }

    #[test]
    #[should_panic(expected = "no `.ftl` files found for locale")]
    fn missing_locale_panics_with_both_expected_layouts() {
        let dir = tempfile::tempdir().unwrap();
        parse_locale_messages(dir.path(), "en");
    }
}
