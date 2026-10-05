![guinea-devtools-protocol](https://raw.githubusercontent.com/guinea-rs/guinea-plugins/master/assets/banners/guinea-devtools-protocol.png)

What a guinea application and [guinea devtools](../../tools/devtools) say to
each other, and where.

An application connects to devtools, not the other way round: devtools are
started once and applications come and go. The connection is an ogurpchik
session - a named pipe or a unix socket, opened only by a peer that knows the
key devtools wrote when they started - and every message is one `Report` as
JSON inside a `Peer.send` call. Devtools answer the same way with a `Command`,
and only with one the other side listed among its `Capability`s.

Both ends are built from this crate, so a message that does not parse is an
error rather than skipped. What a newer peer sends and this one cannot name -
a report, a trace point, a capability - reads as `Unknown`, and the rest of
the message still reads.

`PROTOCOL` is the version both ends compare when they meet.

The application's side is [`guinea-plugin-devtools`](../devtools); a native
inspector, like the XAML tap devtools inject into a WinUI application, speaks
the same protocol.
