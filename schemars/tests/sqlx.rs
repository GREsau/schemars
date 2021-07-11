mod util;
use sqlx::types::Json;
use util::*;

#[test]
fn sqlx() -> TestResult {
    test_default_generated_schema::<Json<String>>("sqlx")
}
