mod util;
use smol_str01::SmolStr;
use util::*;

#[test]
fn smol_str01() -> TestResult {
    test_default_generated_schema::<SmolStr>("smol_str01")
}
