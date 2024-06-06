mod util;
use util::*;

#[test]
fn smol_str01() -> TestResult {
    test_default_generated_schema::<smol_str01::SmolStr>("smol_str01")
}

#[test]
fn smol_str02() -> TestResult {
    test_default_generated_schema::<smol_str02::SmolStr>("smol_str02")
}
