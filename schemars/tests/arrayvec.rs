mod util;
use util::*;

#[test]
fn arrayvec07() -> TestResult {
    test_default_generated_schema::<arrayvec07::ArrayVec<i32, 16>>("arrayvec")
}

#[test]
fn arrayvec07_string() -> TestResult {
    test_default_generated_schema::<arrayvec07::ArrayString<16>>("arrayvec_string")
}
