use crate::prelude::*;
use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(JsonSchema, Deserialize, Serialize)]
struct Map {
    inner: HashMap<Key, String>,
}

#[derive(Debug, JsonSchema, Deserialize, Serialize, Eq, PartialEq, Hash, Copy, Clone)]
#[serde(deny_unknown_fields, rename_all = "kebab-case", into = "&str")]
enum Key {
    A,
    B,
}

impl From<Key> for &str {
    fn from(value: Key) -> Self {
        match value {
            Key::A => "a",
            Key::B => "b",
        }
    }
}
impl From<&str> for Key {
    fn from(value: &str) -> Self {
        match value {
            "a" => Key::A,
            "b" => Key::B,
            _ => unreachable!(),
        }
    }
}

#[test]
fn hashmap_with_enum_key() {
    test!(Map).assert_snapshot();
}
