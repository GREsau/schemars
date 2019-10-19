use schemars::{schema_for, JsonSchema};
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
    let schema = schema_for!(MyStruct);
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
