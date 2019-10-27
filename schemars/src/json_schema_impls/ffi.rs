use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::ffi::{CStr, CString, OsStr, OsString};

impl JsonSchema for OsString {
    fn schema_name() -> String {
        "OsString".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut unix_schema = SchemaObject::default();
        unix_schema.instance_type = Some(InstanceType::Object.into());
        unix_schema.object().required.insert("Unix".to_owned());
        unix_schema
            .object()
            .properties
            .insert("Unix".to_owned(), <Vec<u8>>::json_schema(gen));

        let mut win_schema = SchemaObject::default();
        win_schema.instance_type = Some(InstanceType::Object.into());
        win_schema.object().required.insert("Windows".to_owned());
        win_schema
            .object()
            .properties
            .insert("Windows".to_owned(), <Vec<u16>>::json_schema(gen));

        let mut schema = SchemaObject::default();
        schema.subschemas().one_of = Some(vec![unix_schema.into(), win_schema.into()]);
        schema.into()
    }
}

impl JsonSchema for OsStr {
    fn schema_name() -> String {
        <OsString>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <OsString>::json_schema(gen)
    }
}

impl JsonSchema for CString {
    no_ref_schema!();

    fn schema_name() -> String {
        <Vec<u8>>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <Vec<u8>>::json_schema(gen)
    }
}

impl JsonSchema for CStr {
    no_ref_schema!();

    fn schema_name() -> String {
        <Vec<u8>>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <Vec<u8>>::json_schema(gen)
    }
}
