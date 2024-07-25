mod util;
use compact_str::CompactString;
use util::*;

#[test]
fn smol_str() -> TestResult {
    test_default_generated_schema::<CompactString>("compact_str")
}
