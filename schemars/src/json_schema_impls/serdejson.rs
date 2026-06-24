use crate::SchemaGenerator;
use crate::_alloc_prelude::*;
#[cfg(not(feature = "serde_json_arbitrary_precision"))]
use crate::json_schema;
use crate::{JsonSchema, Schema};
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
#[cfg(not(feature = "serde_json_arbitrary_precision"))]
use serde_json::Number;
use serde_json::{Map, Value};

impl JsonSchema for Value {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "AnyValue".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        true.into()
    }
}

forward_impl!(Map<String, Value> => BTreeMap<String, Value>);

#[cfg(not(feature = "serde_json_arbitrary_precision"))]
impl JsonSchema for Number {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "Number".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "number"
        })
    }
}

#[cfg(feature = "raw_value")]
forward_impl!(serde_json::value::RawValue => Value);
