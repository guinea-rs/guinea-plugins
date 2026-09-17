fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/devtools.capnp")
        .run()
        .expect("compiling schema/devtools.capnp needs the `capnp` binary on PATH");

    println!("cargo:rerun-if-changed=schema/devtools.capnp");
}
