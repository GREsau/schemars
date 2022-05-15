mod util;
use rocket::fs::TempFile;
use util::*;

#[test]
fn tempfile() -> TestResult {
    test_default_generated_schema::<TempFile>("rocket_tempfile")
}