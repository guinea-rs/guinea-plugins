use guinea_devtools_protocol::{
    Actor, BusSubscription, CHANGES_SINCE, Changes, PROTOCOL, Root, Snapshot, Timer, takes_changes,
};
use ogurpchik::auth::handshake::Version;

#[test]
fn changes_go_to_devtools_new_enough_to_take_them() {
    let version = |minor| Version {
        major: 3,
        minor,
        patch: 0,
    };

    assert!(!takes_changes(version(0)));
    assert!(takes_changes(version(1)));
    assert!(takes_changes(version(4)));
    assert!(
        takes_changes(PROTOCOL.version),
        "this version takes its own"
    );
    assert!(CHANGES_SINCE <= PROTOCOL.version);
}

#[test]
fn changes_bring_a_snapshot_up_to_date() {
    let actor = |id, state: &str| Actor {
        id,
        state: state.into(),
        ..Actor::default()
    };
    let root = |id, route: &str| Root {
        id,
        route: Some(route.into()),
        ..Root::default()
    };
    let mut snapshot = Snapshot {
        at: 1,
        actors: vec![actor(1, "old"), actor(2, "kept"), actor(3, "gone")],
        roots: vec![root(10, "/a"), root(11, "/b")],
        global_bus: vec![BusSubscription {
            event: "a::Old".into(),
            subscribers: 1,
        }],
        ..Snapshot::default()
    };

    snapshot.apply(Changes {
        at: 9,
        actors: vec![actor(1, "new"), actor(4, "born")],
        actors_gone: vec![3],
        roots: vec![root(11, "/c")],
        roots_gone: vec![10],
        timers: Some(vec![Timer {
            id: 5,
            ..Timer::default()
        }]),
        ..Changes::default()
    });

    let states: Vec<(u64, &str)> = snapshot
        .actors
        .iter()
        .map(|actor| (actor.id, actor.state.as_str()))
        .collect();
    assert_eq!(states, [(1, "new"), (2, "kept"), (4, "born")]);
    let routes: Vec<(u64, Option<&str>)> = snapshot
        .roots
        .iter()
        .map(|root| (root.id, root.route.as_deref()))
        .collect();
    assert_eq!(routes, [(11, Some("/c"))]);
    assert_eq!(snapshot.timers.len(), 1);
    assert_eq!(
        snapshot.global_bus.len(),
        1,
        "a list that did not come is kept"
    );
    assert_eq!(snapshot.at, 9);
}

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
