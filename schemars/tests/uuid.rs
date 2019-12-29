mod util;
use util::*;
use uuid::Uuid;

#[test]
fn uuid() -> TestResult {
    test_default_generated_schema::<Uuid>("uuid")
}
