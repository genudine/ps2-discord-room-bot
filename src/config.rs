use lazy_static::lazy_static;
use std::collections::HashMap;
use tracing::log::trace;

lazy_static! {
  // Make a hashmap from the env var WATCH_CHANNELS in the format of "server_id:channel_id"
    pub static ref WATCH_CHANNELS: HashMap<String, String> = {
        let mut m = HashMap::new();
        for (key, value) in std::env::vars() {
            if key.starts_with("WATCH_CHANNEL") {
                let mut split = value.split(":");
                let server_id = split.next().unwrap();
                let channel_id = split.next().unwrap();
                m.insert(server_id.to_string(), channel_id.to_string());
            }
        }
        trace!("Watch channels: {:?}", m);
        m
    };
}
