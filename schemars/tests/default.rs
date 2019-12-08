mod util;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use util::*;

fn ten_and_true() -> MyStruct2 {
    MyStruct2 {
        my_int: 10,
        my_bool: true,
    }
}

fn six() -> i32 {
    6
}

fn custom_serialize<S>(value: &MyStruct2, ser: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    ser.collect_str(&format_args!("i:{} b:{}", value.my_int, value.my_bool))
}

#[derive(Default, Deserialize, Serialize, JsonSchema, Debug)]
#[serde(default)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    #[serde(serialize_with = "custom_serialize")]
    pub my_struct2: MyStruct2,
}

#[derive(Default, Deserialize, Serialize, JsonSchema, Debug)]
#[serde(default = "ten_and_true")]
pub struct MyStruct2 {
    #[serde(default = "six")]
    pub my_int: i32,
    pub my_bool: bool,
}

#[test]
fn schema_default_values() -> TestResult {
    test_default_generated_schema::<MyStruct>("default")
}
