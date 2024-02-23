mod util;
use util::*;

#[test]
fn vec1() -> TestResult {
    test_default_generated_schema::<vec1::Vec1<i32>>("vec1")
}
