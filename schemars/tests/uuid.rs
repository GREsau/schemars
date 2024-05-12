mod util;
use util::*;

#[test]
fn uuid1() -> TestResult {
    test_default_generated_schema::<uuid1::Uuid>("uuid")
}
