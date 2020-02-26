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
        // TODO use add_schema_property
        unix_schema.object().required.insert("Unix".to_owned());
        unix_schema
            .object()
            .properties
            .insert("Unix".to_owned(), <Vec<u8>>::json_schema(gen));

        let mut win_schema = SchemaObject::default();
        win_schema.instance_type = Some(InstanceType::Object.into());
        // TODO use add_schema_property
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

forward_impl!(OsStr => OsString);

forward_impl!(CString => Vec<u8>);
forward_impl!(CStr => Vec<u8>);
