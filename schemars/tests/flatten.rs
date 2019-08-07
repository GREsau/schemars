use pretty_assertions::assert_eq;
use schemars::{schema::*, schema_for, MakeSchema};
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
fn flatten_schema() -> Result<(), Box<dyn Error>> {
    let flat = schema_for!(Flat)?;
    let mut deep = schema_for!(Deep1)?;
    match deep {
        Schema::Object(ref mut o) => o.title = Some("Flat".to_owned()),
        _ => assert!(false, "Schema was not object: {:?}", deep),
    };
    assert_eq!(flat, deep);
    Ok(())
}
