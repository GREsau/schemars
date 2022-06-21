mod util;
use util::*;

#[test]
fn ulid() -> TestResult {
    test_default_generated_schema::<ulid::Ulid>("ulid")
}
