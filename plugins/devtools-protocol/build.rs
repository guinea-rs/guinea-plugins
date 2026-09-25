use std::path::Path;

const SCHEMA: &str = "schema/devtools.capnp";

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file(SCHEMA)
        .run()
        .expect("compiling schema/devtools.capnp needs the `capnp` binary on PATH");

    let id = format!(
        "/// Identity of `{SCHEMA}`, handed to the ogurpchik handshake: a plugin and\n\
         /// devtools built from different revisions of it refuse each other rather\n\
         /// than misread each other.\n\
         pub const DEVTOOLS_SCHEMA_ID: u64 = {:#018x};\n",
        schema_hash(SCHEMA)
    );

    let out = std::env::var("OUT_DIR").expect("cargo sets OUT_DIR for build scripts");
    std::fs::write(Path::new(&out).join("schema_id.rs"), id).expect("writing the schema id");

    println!("cargo:rerun-if-changed={SCHEMA}");
}

/// FNV-1a over the schema's bytes, with CRLF read as LF so a checkout on
/// Windows and one in WSL agree.
fn schema_hash(path: &str) -> u64 {
    let text = std::fs::read_to_string(path).expect("reading the schema");

    text.replace("\r\n", "\n")
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}
