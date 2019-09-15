# schemars
Generate JSON Schema documents from Rust code

Work in progress!

## Basic Usage

```rust
use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct MyStruct {
    my_int: i32,
    my_nullable: Option<bool>,
    my_nested_struct: Nested,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct Nested {
    #[serde(default)]
    my_string: String,
    #[serde(rename = "myArray")]
    my_float_vec: Vec<f32>,
    my_recursive_struct: Option<Box<Nested>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = schema_for!(MyStruct)?;
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
```

This outputs the following:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MyStruct",
  "type": "object",
  "definitions": {
    "Nested": {
      "type": "object",
      "required": [
        "myArray",
        "myRecursiveStruct"
      ],
      "properties": {
        "myArray": {
          "type": "array",
          "items": {
            "type": "number",
            "format": "float"
          }
        },
        "myRecursiveStruct": {
          "anyOf": [
            {
              "$ref": "#/definitions/Nested"
            },
            {
              "type": "null"
            }
          ]
        },
        "myString": {
          "type": "string"
        }
      }
    }
  },
  "required": [
    "myInt",
    "myNestedStruct",
    "myNullable"
  ],
  "properties": {
    "myInt": {
      "type": "integer",
      "format": "int32"
    },
    "myNestedStruct": {
      "$ref": "#/definitions/Nested"
    },
    "myNullable": {
      "type": [
        "boolean",
        "null"
      ]
    }
  }
}
```

Note that the `#[serde(...)]` attributes are respected.
