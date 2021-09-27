use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use mac_address::MacAddress;

impl JsonSchema for MacAddress {
    fn schema_name() -> String {
        "MacAddress".to_string()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            metadata: Some(Box::new(Metadata {
                description: Some("Contains the individual bytes of the MAC address.".to_string()),
                examples: vec![serde_json::to_value(
                    "aa:bb:cc:00:11:22".parse::<MacAddress>().unwrap(),
                )]
                .into_iter()
                .flatten()
                .collect(),
                ..Default::default()
            })),
            instance_type: Some(InstanceType::String.into()),
            format: Some("mac_addr".to_owned()),
            ..Default::default()
        }
        .into()
    }
}
