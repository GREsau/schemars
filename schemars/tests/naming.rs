use pretty_assertions::assert_eq;
use schemars::{schema_for, MakeSchema};
use serde::{Deserialize, Serialize};
use std::error::Error;

const EXPECTED: &str = r#"
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MyStruct_For_Integer_And_Null_And_Boolean_And_Array_Of_String",
  "type": "object",
  "properties": {
    "float": {
      "type": "number"
    },
    "t": {
      "type": "integer"
    },
    "u": {
      "type": "null"
    },
    "v": {
      "type": "boolean"
    },
    "w": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
"#;

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
struct MyStruct<T, U, V, W> {
    t: T,
    u: U,
    float: f32,
    v: V,
    w: W,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
#[serde(rename = "MyStruct")]
struct MyRenamedStruct<T, U, V, W> {
    t: T,
    u: U,
    float: f32,
    v: V,
    w: W,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, MakeSchema)]
#[serde(remote = "MyStruct")]
struct MyRemoteStruct<T, U, V, W> {
    t: T,
    u: U,
    float: f32,
    v: V,
    w: W,
}

#[test]
fn default_name_multiple_type_params() -> Result<(), Box<dyn Error>> {
    let actual = schema_for!(MyStruct<i32, (), bool, Vec<String>>)?;
    let expected = serde_json::from_str(EXPECTED)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
#[ignore] // not yet implemented
fn overriden_with_rename_name_multiple_type_params() -> Result<(), Box<dyn Error>> {
    let actual = schema_for!(MyRenamedStruct<i32, (), bool, Vec<String>>)?;
    let expected = serde_json::from_str(EXPECTED)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
#[ignore] // not yet implemented
fn overriden_with_remote_name_multiple_type_params() -> Result<(), Box<dyn Error>> {
    let actual = schema_for!(MyRemoteStruct<i32, (), bool, Vec<String>>)?;
    let expected = serde_json::from_str(EXPECTED)?;
    assert_eq!(actual, expected);
    Ok(())
}
