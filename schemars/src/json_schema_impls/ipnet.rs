use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use ipnet::{IpNet, Ipv4Net, Ipv6Net};

impl JsonSchema for IpNet {
    no_ref_schema!();

    fn schema_name() -> String {
        "IpNet".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ipnet".to_owned()),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for Ipv4Net {
    no_ref_schema!();

    fn schema_name() -> String {
        "Ipv4Net".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ipv4net".to_owned()),
            ..Default::default()
        }
        .into()
    }
}

impl JsonSchema for Ipv6Net {
    no_ref_schema!();

    fn schema_name() -> String {
        "Ipv6Net".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ipv6net".to_owned()),
            ..Default::default()
        }
        .into()
    }
}
