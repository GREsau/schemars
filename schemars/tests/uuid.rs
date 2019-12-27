mod util;
use uuid::Uuid;
use util::*;

#[test]
fn uuid() -> TestResult {
    test_default_generated_schema::<Uuid>("uuid")
}
