#[test]
fn the_protocol_is_named_after_its_schema() {
    let schema = include_str!("../schema/devtools.capnp");
    let declared = schema
        .lines()
        .find_map(|line| line.trim().strip_prefix("@0x")?.strip_suffix(';'))
        .expect("the schema declares its id");

    assert_eq!(
        u64::from_str_radix(declared, 16).expect("a hex id"),
        guinea_devtools_protocol::PROTOCOL.id
    );
}
