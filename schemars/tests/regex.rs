mod util;
use regex::Regex;
use util::*;

#[test]
fn regex() -> TestResult {
    test_default_generated_schema::<Regex>("regex")
}
