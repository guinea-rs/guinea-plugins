@0x96fa2dd107e3d402;

# Both ends serve this. An application sends reports; devtools will send
# commands the same way. What a message means is in `json`, as the Rust types
# in this crate describe it.
interface Peer {
  send @0 (json :Text);
}
