use crate::SchemaGenerator;
use crate::_alloc_prelude::*;
use crate::{json_schema, JsonSchema, Schema};
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use ijson::IArray;
use ijson::INumber;
use ijson::IObject;
use ijson::IString;
use ijson::IValue;

impl JsonSchema for IValue {
    inline_schema!();

    fn schema_name() -> Cow<'static, str> {
        "AnyValue".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        true.into()
    }
}

forward_impl!(IObject => BTreeMap<String, IValue>);
forward_impl!(IArray => Vec<IValue>);
forward_impl!(IString => String);

impl JsonSchema for INumber {
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
