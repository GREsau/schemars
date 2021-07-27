mod util;
use ipnetwork::IpNetwork;
use util::*;

#[test]
fn ipnetwork() -> TestResult {
    test_default_generated_schema::<IpNetwork>("ipnetwork")
}