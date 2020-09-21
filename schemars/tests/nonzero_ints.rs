mod util;

use super::*;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
struct MyStruct {
    unsigned: u32,
    nonzero_unsigned: std::num::NonZeroU32,
    signed: i32,
    nonzero_signed: std::num::NonZeroI32,
}

#[test]
fn nonzero_ints() -> TestResult {
    test_default_generated_schema::<MyStruct>("nonzero_ints")
}
