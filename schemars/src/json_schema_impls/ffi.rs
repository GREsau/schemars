use crate::_alloc_prelude::*;
use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use std::ffi::{CStr, CString, OsStr, OsString};

impl JsonSchema for OsString {
    fn schema_name() -> Cow<'static, str> {
        "OsString".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "std::ffi::OsString".into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "Unix": <Vec<u8>>::json_schema(gen)
                    },
                    "required": ["Unix"]
                },
                {
                    "type": "object",
                    "properties": {
                        "Windows": <Vec<u16>>::json_schema(gen)
                    },
                    "required": ["Windows"]
                },
            ]
        })
    }
}

forward_impl!(OsStr => OsString);

forward_impl!(CString => Vec<u8>);
forward_impl!(CStr => Vec<u8>);
