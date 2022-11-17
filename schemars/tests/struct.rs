mod util;
use schemars::{gen::SchemaSettings, JsonSchema};
use util::*;

// Ensure that schemars_derive uses the full path to std::string::String
pub struct String;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct Struct {
    foo: i32,
    bar: bool,
    baz: Option<&'static str>,
}

#[test]
fn struct_normal() -> TestResult {
    test_default_generated_schema::<Struct>("struct-normal")
}

#[derive(JsonSchema)]
pub struct Tuple(i32, bool, Option<&'static str>);

#[test]
fn struct_tuple() -> TestResult {
    test_default_generated_schema::<Tuple>("struct-tuple")
}

#[derive(JsonSchema)]
pub struct Newtype(i32);

#[test]
fn struct_newtype() -> TestResult {
    test_default_generated_schema::<Newtype>("struct-newtype")
}

#[derive(JsonSchema)]
pub struct Unit;

#[test]
fn struct_unit() -> TestResult {
    test_default_generated_schema::<Unit>("struct-unit")
}

#[allow(dead_code)]
#[derive(JsonSchema)]
pub struct RecursiveStruct {
    foo: Vec<RecursiveStruct>,
}

#[test]
fn struct_recursive() -> TestResult {
    test_default_generated_schema::<RecursiveStruct>("struct-recursive")
}

#[test]
fn struct_recursive_strict_inline() -> TestResult {
    let mut settings = SchemaSettings::default();
    settings.strict_inline_subschemas = true;
    test_generated_schema::<RecursiveStruct>("struct-recursive-strict-inline", settings)
}
