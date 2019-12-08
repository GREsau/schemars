mod util;
use pretty_assertions::assert_eq;
use schemars::{schema_for, JsonSchema};

#[derive(Debug, JsonSchema)]
struct Flat {
    f: f32,
    b: bool,
    s: String,
    #[serde(default)]
    os: String,
    v: Vec<i32>,
}

#[derive(Debug, JsonSchema)]
#[schemars(rename = "Flat")]
struct Deep1 {
    f: f32,
    #[schemars(flatten)]
    deep2: Deep2,
    v: Vec<i32>,
}

#[allow(clippy::option_option)]
#[derive(Debug, JsonSchema)]
struct Deep2 {
    b: bool,
    #[serde(flatten)]
    deep3: Deep3,
    #[serde(flatten)]
    deep4: Box<Option<Option<Box<Deep4>>>>,
}

#[derive(Debug, JsonSchema)]
struct Deep3 {
    s: &'static str,
}

#[derive(Debug, JsonSchema)]
struct Deep4 {
    #[serde(default)]
    os: &'static str,
}

#[test]
fn flatten_schema() {
    let flat = schema_for!(Flat);
    let deep = schema_for!(Deep1);
    assert_eq!(flat, deep);
}
