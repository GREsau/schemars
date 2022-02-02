use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use serde_json::json;
use ipnetwork::{Ipv4Network, Ipv6Network};

impl JsonSchema for Ipv4Network {
    no_ref_schema!();

    fn schema_name() -> String {
        "Ipv4Network".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                // Rough approximation keeping regexp simple
                pattern: Some(r"^[12]?[0-9]?[0-9]\.[12]?[0-9]?[0-9]\.[12]?[0-9]?[0-9]\.[12]?[0-9]?[0-9]/[1-3]?[0-9]$".to_string()),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                // Includes examples both with zero and non-zero host parts, with more common zero part first.
                examples: vec!["1.2.3.0/24".into(), "1.2.3.4/24".into(), "1.2.3.4/32".into(), "0.0.0.0/0".into()],
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for Ipv6Network {
    no_ref_schema!();

    fn schema_name() -> String {
        "Ipv6Network".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                // Very rough approximation. IPv4 mapped/embedded addresses make generic regexp really hard
                // (see https://datatracker.ietf.org/doc/html/rfc5952).
                pattern: Some(r"^[0-9a-f:.]+/1?[0-9][0-9]?$".to_string()),
                ..Default::default()
            })),
            metadata: Some(Box::new(Metadata {
                // Includes examples both with zero and non-zero host parts, with more common zero part first.
                examples: vec!["2001:db8::0/64".into(), "2001:db8::1/64".into(), "2001:db8::1/128".into(), "::ffff:192.0.2.1/128".into(), "::/0".into()],
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
