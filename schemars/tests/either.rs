mod util;
use either1::Either;
use util::*;

#[test]
fn either() -> TestResult {
    test_default_generated_schema::<Either<i32, Either<bool, ()>>>("either")
}
