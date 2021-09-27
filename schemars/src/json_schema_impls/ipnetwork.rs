use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};

impl JsonSchema for IpNetwork {
    fn schema_name() -> String {
        "IpNetwork".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some("Represents a generic network range. This type can have two variants: the v4 and the v6 case.".to_string()),
                examples: vec![
                    serde_json::to_value(IpNetwork::V4("192.168.0.0/24".parse().unwrap())),
                    serde_json::to_value(IpNetwork::V6("fc00::/7".parse().unwrap())),
                ].into_iter().flatten().collect(),
                ..Default::default()
            })),
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![
                    gen.subschema_for::<Ipv4Network>(),
                    gen.subschema_for::<Ipv6Network>(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for Ipv4Network {
    fn schema_name() -> String {
        "Ipv4Network".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "Represents a network range where the IP addresses are of v4".to_string(),
                ),
                examples: vec![serde_json::to_value(
                    "192.168.0.0/24".parse::<Ipv4Network>().unwrap(),
                )]
                .into_iter()
                .flatten()
                .collect(),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: Some("ipv4-cidr".to_string()),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for Ipv6Network {
    fn schema_name() -> String {
        "Ipv6Network".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some(
                    "Represents a network range where the IP addresses are of v6".to_string(),
                ),
                examples: vec![serde_json::to_value(
                    "fc00::/7".parse::<Ipv6Network>().unwrap(),
                )]
                .into_iter()
                .flatten()
                .collect(),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: Some("ipv6-cidr".to_string()),
            ..Default::default()
        }
        .into()
    }
}
