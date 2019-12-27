mod util;

use schemars::JsonSchema;
use serde::{Serialize};
use util::*;
use std::collections::{HashMap, HashSet};

enum Or<A, B> {
    #[allow(dead_code)]
    A(A),
    #[allow(dead_code)]
    B(B),
}

#[derive(JsonSchema, Serialize)]
#[serde(untagged, remote = "Or")]
enum OrDef<A, B> {
    A(A),
    B(B),
}

struct Str<'a>(&'a str);

#[derive(JsonSchema, Serialize)]
#[serde(remote = "Str")]
struct StrDef<'a>(&'a str);

#[derive(JsonSchema, Serialize)]
struct MyStruct<'a, T: Serialize> {
    // #[serde(with = "OrDef::<_, _>")]
    // byte_or_bool1: Or<u8, bool>,
    #[serde(with = "OrDef::<u8, bool>")]
    byte_or_bool2: Or<u8, bool>,
    // #[serde(with = "OrDef::<_, _>")]
    // unit_or_t1: Or<(), T>,
    #[serde(with = "OrDef::<(), T>")]
    unit_or_t2: Or<(), T>,
    #[serde(borrow, with = "StrDef")]
    s: Str<'a>,
    // #[schemars(with = "HashMap::<_, HashSet<_>>")]
    // map: BTreeMap<String, BTreeSet<String>>,
    #[schemars(with = "HashMap::<String, HashSet<String>>")]
    fake_map: (),
}

#[test]
fn remote_derive_json_schema() -> TestResult {
    test_default_generated_schema::<MyStruct<i32>>("remote_derive_generic")
}
