mod util;
use util::*;

#[test]
fn ipv4network() -> TestResult {
    test_default_generated_schema::<ipnetwork::Ipv4Network>("ipv4network")
}

#[test]
fn ipv6network() -> TestResult {
    test_default_generated_schema::<ipnetwork::Ipv6Network>("ipv6network")
}

#[test]
fn ipnetwork() -> TestResult {
    test_default_generated_schema::<ipnetwork::IpNetwork>("ipnetwork")
}
