mod util;
use util::*;

#[test]
fn arbitrary_int_u1() -> TestResult {
    test_default_generated_schema::<arbitrary_int::u1>("arbitrary_int_u1")
}
#[test]
fn arbitrary_int_u9() -> TestResult {
    test_default_generated_schema::<arbitrary_int::u9>("arbitrary_int_u9")
}
#[test]
fn arbitrary_int_u17() -> TestResult {
    test_default_generated_schema::<arbitrary_int::u17>("arbitrary_int_u17")
}
#[test]
fn arbitrary_int_u33() -> TestResult {
    test_default_generated_schema::<arbitrary_int::u33>("arbitrary_int_u33")
}
#[test]
fn arbitrary_int_u65() -> TestResult {
    test_default_generated_schema::<arbitrary_int::u65>("arbitrary_int_u65")
}
#[test]
fn arbitrary_int_u127() -> TestResult {
    test_default_generated_schema::<arbitrary_int::u127>("arbitrary_int_u127")
}
