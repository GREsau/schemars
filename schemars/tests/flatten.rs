mod util;
use pretty_assertions::assert_eq;
use schemars::{schema_for, JsonSchema};
use util::*;

#[derive(Debug, JsonSchema)]
struct Flat {
    foo: f32,
    bar: bool,
    baz: String,
    foobar: Vec<i32>,
}

#[derive(Debug, JsonSchema)]
#[schemars(rename = "Flat")]
struct Deep1 {
    foo: f32,
    #[serde(flatten)]
    deep2: Deep2,
    foobar: Vec<i32>,
}

#[derive(Debug, JsonSchema)]
struct Deep2 {
    bar: bool,
    #[serde(flatten)]
    deep3: Deep3,
}

#[derive(Debug, JsonSchema)]
struct Deep3 {
    baz: &'static str,
}

#[test]
fn flatten_schema() -> TestResult {
    let flat = schema_for!(Flat)?;
    let deep = schema_for!(Deep1)?;
    assert_eq!(flat, deep);
    Ok(())
}
