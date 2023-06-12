mod util;
use schemars::JsonSchema_repr;
use util::*;

#[derive(JsonSchema_repr)]
#[repr(u8)]
pub enum Enum {
    Zero,
    One,
    Five = 5,
    Six,
    Three = 3,
}

#[test]
fn enum_repr() -> TestResult {
    test_default_generated_schema::<Enum>("enum-repr")
}

#[derive(JsonSchema_repr)]
#[repr(u8)]
#[schemars(extension = "x-enumNames")]
pub enum EnumWithXEnumNames {
    Zero,
    One,
    Five = 5,
    Six,
    Three = 3,
}

#[test]
fn enum_repr_with_x_enum_names() -> TestResult {
    test_default_generated_schema::<EnumWithXEnumNames>("enum-repr-with-x-enum-names")
}

#[derive(JsonSchema_repr)]
#[repr(i64)]
#[serde(rename = "Renamed")]
/// Description from comment
pub enum EnumWithAttrs {
    Zero,
    One,
    Five = 5,
    Six,
    Three = 3,
}

#[test]
fn enum_repr_with_attrs() -> TestResult {
    test_default_generated_schema::<EnumWithAttrs>("enum-repr-with-attrs")
}
