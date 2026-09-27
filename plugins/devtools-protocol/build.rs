const SCHEMA: &str = "schema/devtools.capnp";

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file(SCHEMA)
        .run()
        .expect("compiling schema/devtools.capnp needs the `capnp` binary on PATH");

    println!("cargo:rerun-if-changed={SCHEMA}");
}
