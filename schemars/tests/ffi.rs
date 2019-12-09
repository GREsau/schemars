mod util;
use schemars::JsonSchema;
use std::ffi::{OsStr, OsString};
use util::*;

#[derive(Debug, JsonSchema)]
struct OsStrings {
    owned: OsString,
    borrowed: &'static OsStr,
}

#[test]
fn os_strings() -> TestResult {
    test_default_generated_schema::<OsStrings>("os_strings")
}
