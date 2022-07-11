mod util;
use tinyvec::TinyVec;
use util::*;

#[test]
fn tinyvec() -> TestResult {
    test_default_generated_schema::<TinyVec<[String; 2]>>("tinyvec")
}
