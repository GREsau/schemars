use crate::gen::SchemaGenerator;
use crate::{json_schema, JsonSchema, Schema};
use serde_json::Value;
use std::borrow::Cow;
use std::ops::{Bound, Range, RangeInclusive};

impl<T: JsonSchema> JsonSchema for Option<T> {
    no_ref_schema!();

    fn schema_name() -> Cow<'static, str> {
        format!("Nullable_{}", T::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("Option<{}>", T::schema_id()).into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = gen.subschema_for::<T>();

        if gen.settings().option_add_null_type {
            schema = match schema.try_to_object() {
                Ok(mut obj) => {
                    let instance_type = obj.get_mut("type");
                    match instance_type {
                        Some(Value::Array(array)) => {
                            let null = Value::from("null");
                            if !array.contains(&null) {
                                array.push(null);
                            }
                            obj.into()
                        }
                        Some(Value::String(string)) => {
                            if string != "null" {
                                *instance_type.unwrap() =
                                    Value::Array(vec![std::mem::take(string).into(), "null".into()])
                            }
                            obj.into()
                        }
                        _ => json_schema!({
                            "anyOf": [
                                obj,
                                <()>::json_schema(gen)
                            ]
                        }),
                    }
                }
                Err(true) => true.into(),
                Err(false) => <()>::json_schema(gen),
            }
        }

        if gen.settings().option_nullable {
            schema
                .ensure_object()
                .insert("nullable".into(), true.into());
        };

        schema
    }

    fn _schemars_private_non_optional_json_schema(gen: &mut SchemaGenerator) -> Schema {
        T::_schemars_private_non_optional_json_schema(gen)
    }

    fn _schemars_private_is_option() -> bool {
        true
    }
}

impl<T: JsonSchema, E: JsonSchema> JsonSchema for Result<T, E> {
    fn schema_name() -> Cow<'static, str> {
        format!("Result_of_{}_or_{}", T::schema_name(), E::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("Result<{}, {}>", T::schema_id(), E::schema_id()).into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "Ok": gen.subschema_for::<T>()
                    },
                    "required": ["Ok"]
                },
                {
                    "type": "object",
                    "properties": {
                        "Err": gen.subschema_for::<E>()
                    },
                    "required": ["Err"]
                }
            ]
        })
    }
}

impl<T: JsonSchema> JsonSchema for Bound<T> {
    fn schema_name() -> Cow<'static, str> {
        format!("Bound_of_{}", T::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("Bound<{}>", T::schema_id()).into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "oneOf": [
                {
                    "type": "object",
                    "properties": {
                        "Included": gen.subschema_for::<T>()
                    },
                    "required": ["Included"]
                },
                {
                    "type": "object",
                    "properties": {
                        "Excluded": gen.subschema_for::<T>()
                    },
                    "required": ["Excluded"]
                },
                {
                    "type": "string",
                    "const": "Unbounded"
                }
            ]
        })
    }
}

impl<T: JsonSchema> JsonSchema for Range<T> {
    fn schema_name() -> Cow<'static, str> {
        format!("Range_of_{}", T::schema_name()).into()
    }

    fn schema_id() -> Cow<'static, str> {
        format!("Range<{}>", T::schema_id()).into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let subschema = gen.subschema_for::<T>();
        json_schema!({
            "type": "object",
            "properties": {
                "start": subschema,
                "end": subschema
            },
            "required": ["start", "end"]
        })
    }
}

forward_impl!((<T: JsonSchema> JsonSchema for RangeInclusive<T>) => Range<T>);

forward_impl!((<T: ?Sized> JsonSchema for std::marker::PhantomData<T>) => ());

forward_impl!((<'a> JsonSchema for std::fmt::Arguments<'a>) => String);
