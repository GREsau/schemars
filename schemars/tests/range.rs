mod util;
use schemars::JsonSchema;
use std::ops::{Bound, Range, RangeInclusive};
use util::*;

#[derive(Debug, JsonSchema)]
struct MyStruct {
    range: Range<usize>,
    inclusive: RangeInclusive<f64>,
    bound: Bound<String>,
}

#[test]
fn result() -> TestResult {
    test_default_generated_schema::<MyStruct>("range")
}
