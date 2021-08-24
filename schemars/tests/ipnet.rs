
mod util;
use ipnet::IpNet;
use util::*;

#[test]
fn ipnetwork() -> TestResult {
    test_default_generated_schema::<IPNet>("ipnet")
} 