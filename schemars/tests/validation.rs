use schemars::{
    schema::{RootSchema, Schema},
    schema_for,
    validation::{schema::SchemaValidator, span::Keys, Validate},
    JsonSchema,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Default, Serialize, Deserialize, JsonSchema)]
pub struct SomeStruct {
    pub some_int: i32,
    pub some_bool: bool,
    pub some_inner: SomeInnerStruct,
}

#[derive(Default, JsonSchema, Serialize, Deserialize)]
pub struct SomeInnerStruct {
    pub inner_value: String,
    pub even_deeper_structs: Vec<SomeEvenDeeperStruct>,
}

#[derive(Default, JsonSchema, Serialize, Deserialize)]
pub struct SomeEvenDeeperStruct {
    pub even_deeper_value: String,
}

#[test]
fn asd() {
    let schema_value = json! {
        {
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "SomeStruct",
            "type": "object",
            "required": [
              "some_bool",
              "some_inner",
              "some_int"
            ],
            "additionalProperties": false,
            "properties": {
              "some_bool": {
                "type": "boolean"
              },
              "some_inner": {
                "type": "object",
                "required": [
                  "even_deeper_structs",
                  "inner_value"
                ],
                "properties": {
                  "even_deeper_structs": {
                    "type": "array",
                    "maxItems": 1,
                    "items": {
                        "type": "object",
                        "required": [
                          "even_deeper_value"
                        ],
                        "properties": {
                          "even_deeper_value": {
                            "type": "string"
                          }
                        }
                    }
                  },
                  "inner_value": {
                    "type": "string"
                  }
                }
              },
              "some_int": {
                "type": "integer",
                "format": "int32"
              }
            }
          }
    };

    let bad = json!{
        {
            "some_int": 0,
            "some_bool": false,
            "some_inner": {
              "inner_value": "",
              "even_deeper_structs": ["faszom", {
                  "even_deeper_value": 2
              }]
            },
            "unexpected_property": 2
        }
    };

    // println!("{:#?}", bad);

    let schema = serde_json::from_value::<RootSchema>(schema_value).unwrap();

    let val = SomeStruct::default();

    let valid = Keys::new(&bad).validate(SchemaValidator::new_root(&schema, None));

    if let Err(e) = &valid {
        println!("{:#}", e);
    }
}
