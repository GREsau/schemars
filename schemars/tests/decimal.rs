mod util;
use util::*;

#[test]
fn rust_decimal() -> TestResult {
    test_default_generated_schema::<rust_decimal::Decimal>("rust_decimal")
}

#[test]
fn bigdecimal() -> TestResult {
    test_default_generated_schema::<bigdecimal::BigDecimal>("bigdecimal")
}
