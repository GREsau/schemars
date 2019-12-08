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

#[derive(Default, Deserialize, Serialize, JsonSchema, Debug)]
#[serde(default)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
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
#[ignore] // not yet implemented (https://github.com/GREsau/schemars/issues/6)
fn schema_default_values() -> TestResult {
    test_default_generated_schema::<MyStruct>("default")
}
