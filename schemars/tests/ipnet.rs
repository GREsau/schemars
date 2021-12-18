mod util;
use schemars::JsonSchema;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};
use util::*;

#[derive(Debug, JsonSchema)]
struct IpNetTypes {
    ipnet: IpNet,
    ipv4net: Ipv4Net,
    ipv6net: Ipv6Net,
}

#[test]
fn url_types() -> TestResult {
    test_default_generated_schema::<IpNetTypes>("ipnet")
}

