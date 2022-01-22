mod util;
use schemars::JsonSchema;
use util::*;

fn is_default<T: Default + PartialEq>(value: &T) -> bool {
    value == &T::default()
}

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

#[derive(Default, JsonSchema, Debug)]
#[serde(default)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    #[serde(serialize_with = "custom_serialize")]
    pub my_struct2: MyStruct2,
    #[serde(
        serialize_with = "custom_serialize",
        skip_serializing_if = "is_default"
    )]
    pub my_struct2_default_skipped: MyStruct2,
    pub not_serialize: NotSerialize,
}

#[derive(Default, JsonSchema, Debug, PartialEq)]
#[serde(default = "ten_and_true")]
pub struct MyStruct2 {
    #[serde(default = "six")]
    pub my_int: i32,
    pub my_bool: bool,
}

#[derive(Default, JsonSchema, Debug)]
pub struct NotSerialize;

#[test]
fn schema_default_values() -> TestResult {
    test_default_generated_schema::<MyStruct>("default")
}
