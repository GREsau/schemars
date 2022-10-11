mod util;
use schemars::JsonSchema;
use util::*;
use ipnetwork::{Ipv4Network, Ipv6Network};

#[derive(Debug, JsonSchema)]
struct IpNetworkTypes {
    ipv4_net_field: Ipv4Network,
    ipv6_net_field: Ipv6Network
}

#[test]
fn ipnetwork_types() -> TestResult {
    test_default_generated_schema::<IpNetworkTypes>("ipnetwork-types")
}
