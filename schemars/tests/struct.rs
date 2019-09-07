mod util;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
pub struct Struct {
    foo: i32,
    bar: bool,
}

#[test]
fn struct_normal() -> TestResult {
    test_default_generated_schema::<Struct>("struct-normal")
}

#[derive(Debug, JsonSchema)]
pub struct Tuple(i32, bool);

#[test]
fn struct_tuple() -> TestResult {
    test_default_generated_schema::<Tuple>("struct-tuple")
}

#[derive(Debug, JsonSchema)]
pub struct Newtype(i32);

#[test]
fn struct_newtype() -> TestResult {
    test_default_generated_schema::<Newtype>("struct-newtype")
}

#[derive(Debug, JsonSchema)]
pub struct Unit;

#[test]
fn struct_unit() -> TestResult {
    test_default_generated_schema::<Unit>("struct-unit")
}
