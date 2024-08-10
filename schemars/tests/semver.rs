mod util;
use schemars::JsonSchema;
use semver1::Version;
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
struct SemverTypes {
    version: Version,
}

#[test]
fn semver_types() -> TestResult {
    test_default_generated_schema::<SemverTypes>("semver")
}
