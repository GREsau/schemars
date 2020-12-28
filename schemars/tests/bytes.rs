mod util;
use bytes::Bytes;
use util::*;

#[test]
fn bytes() -> TestResult {
    test_default_generated_schema::<Bytes>("bytes")
}
