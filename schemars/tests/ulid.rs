mod util;
use ulid::Ulid;
use util::*;

#[test]
fn ulid() -> TestResult {
    test_default_generated_schema::<Ulid>("ulid")
}
