mod util;
use arrayvec::{ArrayString, ArrayVec};
use util::*;

#[test]
fn arrayvec() -> TestResult {
    test_default_generated_schema::<ArrayVec<i32, 16>>("arrayvec")
}

#[test]
fn arrayvec_string() -> TestResult {
    test_default_generated_schema::<ArrayString<16>>("arrayvec_string")
}
