use pretty_assertions::assert_eq;
use schemars::{schema_for, MakeSchema};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
struct Flat {
    foo: f32,
    bar: bool,
    baz: String,
    foobar: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
struct Deep1 {
    foo: f32,
    #[serde(flatten)]
    deep2: Deep2,
    foobar: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
struct Deep2 {
    bar: bool,
    #[serde(flatten)]
    deep3: Deep3,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
struct Deep3 {
    baz: String,
}

#[test]
#[ignore = "flattening is not yet implemented"]
fn flatten_schema() -> Result<(), Box<dyn Error>> {
    assert_eq!(schema_for!(Flat)?, schema_for!(Deep1)?);
    Ok(())
}
