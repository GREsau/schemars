mod util;
use mac_address::MacAddress;
use util::*;

#[test]
fn mac_address() -> TestResult {
    test_default_generated_schema::<MacAddress>("mac_address")
}