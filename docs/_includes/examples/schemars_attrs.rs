use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(rename_all = "camelCase")]
pub struct MyStruct {
    #[serde(rename = "thisIsOverridden")]
    #[schemars(rename = "myNumber")]
    pub my_int: i32,
    pub my_bool: bool,
    #[schemars(default)]
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(Deserialize, Serialize, JsonSchema)]
#[schemars(untagged)]
pub enum MyEnum {    
    StringNewType(String),
    StructVariant {
        floats: Vec<f32>,
    }
}

fn main() {
    let schema = schema_for!(MyStruct);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}
