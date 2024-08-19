use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use ipnetwork020::{IpNetwork, Ipv4Network, Ipv6Network};
use std::borrow::Cow;

impl JsonSchema for IpNetwork {
  always_inline!();

  fn schema_name() -> Cow<'static, str> {
    "Ip".into()
  }

  fn schema_id() -> Cow<'static, str> {
    "ipnetwork::Ip".into()
  }

  fn json_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
          "oneOf": [
            {
              "type": "string",
              "format": "ipv4"
            }
            ,{
              "type": "string",
              "format": "ipv6"
            }
          ]
        })
  }
}

impl JsonSchema for Ipv4Network {
  always_inline!();

  fn schema_name() -> Cow<'static, str> {
    "IpV4".into()
  }

  fn schema_id() -> Cow<'static, str> {
    "ipnetwork::IpV4".into()
  }

  fn json_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
          "type": "string",
          "format": "ipv4"
        })
  }
}

impl JsonSchema for Ipv6Network {
  always_inline!();

  fn schema_name() -> Cow<'static, str> {
    "IpV6".into()
  }

  fn schema_id() -> Cow<'static, str> {
    "ipnetwork::Ipv6Network".into()
  }

  fn json_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
          "type": "string",
          "format": "ipv6"
        })
  }
}