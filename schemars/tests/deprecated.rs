#![allow(deprecated)]

mod util;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
#[deprecated]
pub struct DeprecatedStruct {
    foo: i32,
    #[deprecated]
    deprecated_field: bool,
}

#[test]
fn deprecated_struct() -> TestResult {
    test_default_generated_schema::<DeprecatedStruct>("deprecated-struct")
}

#[derive(Debug, JsonSchema)]
#[deprecated]
pub enum DeprecatedEnum {
    Unit,
    #[deprecated]
    DeprecatedUnitVariant,
    #[deprecated]
    DeprecatedStructVariant {
        foo: i32,
        #[deprecated]
        deprecated_field: bool,
    },
}

#[test]
fn deprecated_enum() -> TestResult {
    test_default_generated_schema::<DeprecatedEnum>("deprecated-enum")
}
