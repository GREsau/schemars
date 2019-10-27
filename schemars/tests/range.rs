mod util;
use schemars::JsonSchema;
use util::*;
use std::ops::{Range, RangeInclusive};

#[derive(Debug, JsonSchema)]
struct MyStruct {
    range: Range<usize>,
    inclusive: RangeInclusive<f64>,
}

#[test]
fn result() -> TestResult {
    test_default_generated_schema::<MyStruct>("range")
}
