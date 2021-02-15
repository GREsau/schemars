mod util;
use semver::Version;
use util::*;

#[test]
fn semver() -> TestResult {
    test_default_generated_schema::<Version>("semver")
}
