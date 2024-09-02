#![no_std]

mod util;
use schemars::JsonSchema;
use util::*;

extern crate alloc as test_alloc;

#[derive(JsonSchema)]
pub struct MyStruct {
    pub my_int: i32,
    pub my_bool: bool,
    pub my_nullable_enum: Option<MyEnum>,
}

#[derive(JsonSchema)]
pub enum MyEnum {
    StringNewType(test_alloc::string::String),
    StructVariant { floats: test_alloc::vec::Vec<f32> },
}

#[test]
fn no_std() -> TestResult {
    test_default_generated_schema::<MyStruct>("no_std")
}
