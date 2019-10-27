use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use serde_json::json;
use std::ops::{Range, RangeInclusive};

impl<T: JsonSchema> JsonSchema for Option<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Nullable_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = if gen.settings().option_nullable {
            T::json_schema(gen)
        } else {
            gen.subschema_for::<T>()
        };
        if gen.settings().option_add_null_type {
            schema = match schema {
                Schema::Bool(true) => Schema::Bool(true),
                Schema::Bool(false) => <()>::json_schema(gen),
                Schema::Object(
                    obj @ SchemaObject {
                        instance_type: Some(_),
                        ..
                    },
                ) => Schema::Object(with_null_type(obj)),
                schema => SchemaObject {
                    subschemas: Some(Box::new(SubschemaValidation {
                        any_of: Some(vec![schema, <()>::json_schema(gen)]),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into(),
            }
        }
        if gen.settings().option_nullable {
            let mut schema_obj: SchemaObject = schema.into();
            schema_obj
                .extensions
                .insert("nullable".to_owned(), json!(true));
            schema = Schema::Object(schema_obj);
        };
        schema
    }

    fn json_schema_optional(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = T::json_schema_optional(gen);
        if let Schema::Object(SchemaObject {
            object: Some(ref mut object_validation),
            ..
        }) = schema
        {
            object_validation.required.clear();
        }
        schema
    }
}

fn with_null_type(mut obj: SchemaObject) -> SchemaObject {
    match obj
        .instance_type
        .as_mut()
        .expect("checked in calling function")
    {
        SingleOrVec::Single(ty) if **ty == InstanceType::Null => {}
        SingleOrVec::Vec(ty) if ty.contains(&InstanceType::Null) => {}
        SingleOrVec::Single(ty) => obj.instance_type = Some(vec![**ty, InstanceType::Null].into()),
        SingleOrVec::Vec(ref mut ty) => ty.push(InstanceType::Null),
    };
    obj
}

impl<T: JsonSchema, E: JsonSchema> JsonSchema for Result<T, E> {
    fn schema_name() -> String {
        format!("Result_Of_{}_Or_{}", T::schema_name(), E::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut ok_schema = SchemaObject::default();
        ok_schema.instance_type = Some(InstanceType::Object.into());
        let obj = ok_schema.object();
        obj.required.insert("Ok".to_owned());
        obj.properties
            .insert("Ok".to_owned(), gen.subschema_for::<T>());

        let mut err_schema = SchemaObject::default();
        err_schema.instance_type = Some(InstanceType::Object.into());
        let obj = err_schema.object();
        obj.required.insert("Err".to_owned());
        obj.properties
            .insert("Err".to_owned(), gen.subschema_for::<E>());

        let mut schema = SchemaObject::default();
        schema.subschemas().one_of = Some(vec![ok_schema.into(), err_schema.into()]);
        schema.into()
    }
}

impl<T: JsonSchema> JsonSchema for Range<T> {
    fn schema_name() -> String {
        format!("Range_Of_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject::default();
        schema.instance_type = Some(InstanceType::Object.into());
        let obj = schema.object();
        obj.required.insert("start".to_owned());
        obj.required.insert("end".to_owned());
        obj.properties
            .insert("start".to_owned(), gen.subschema_for::<T>());
        obj.properties
            .insert("end".to_owned(), gen.subschema_for::<T>());
        schema.into()
    }
}

impl<T: JsonSchema> JsonSchema for RangeInclusive<T> {
    fn schema_name() -> String {
        <Range<T>>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <Range<T>>::json_schema(gen)
    }
}

impl<T: ?Sized> JsonSchema for std::marker::PhantomData<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        <()>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <()>::json_schema(gen)
    }
}

impl<'a> JsonSchema for std::fmt::Arguments<'a> {
    no_ref_schema!();

    fn schema_name() -> String {
        <String>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        <String>::json_schema(gen)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gen::*;
    use crate::tests::{custom_schema_object_for, schema_for, schema_object_for};
    use pretty_assertions::assert_eq;

    #[test]
    fn schema_for_option() {
        let schema = schema_object_for::<Option<i32>>();
        assert_eq!(
            schema.instance_type,
            Some(vec![InstanceType::Integer, InstanceType::Null].into())
        );
        assert_eq!(schema.extensions.get("nullable"), None);
        assert_eq!(schema.subschemas.is_none(), true);
    }

    #[test]
    fn schema_for_option_with_ref() {
        use crate as schemars;
        #[derive(JsonSchema)]
        struct Foo;

        let schema = schema_object_for::<Option<Foo>>();
        assert_eq!(schema.instance_type, None);
        assert_eq!(schema.extensions.get("nullable"), None);
        assert_eq!(schema.subschemas.is_some(), true);
        let any_of = schema.subschemas.unwrap().any_of.unwrap();
        assert_eq!(any_of.len(), 2);
        assert_eq!(any_of[0], Schema::new_ref("#/definitions/Foo".to_string()));
        assert_eq!(any_of[1], schema_for::<()>());
    }

    #[test]
    fn schema_for_option_with_nullable() {
        let settings = SchemaSettings {
            option_nullable: true,
            option_add_null_type: false,
            ..Default::default()
        };
        let schema = custom_schema_object_for::<Option<i32>>(settings);
        assert_eq!(
            schema.instance_type,
            Some(SingleOrVec::from(InstanceType::Integer))
        );
        assert_eq!(schema.extensions.get("nullable"), Some(&json!(true)));
        assert_eq!(schema.subschemas.is_none(), true);
    }

    #[test]
    fn schema_for_result() {
        let schema = schema_object_for::<Result<bool, String>>();
        let one_of = schema.subschemas.unwrap().one_of.unwrap();
        assert_eq!(one_of.len(), 2);

        let ok_schema: SchemaObject = one_of[0].clone().into();
        let obj = ok_schema.object.unwrap();
        assert!(obj.required.contains("Ok"));
        assert_eq!(obj.properties["Ok"], schema_for::<bool>());

        let err_schema: SchemaObject = one_of[1].clone().into();
        let obj = err_schema.object.unwrap();
        assert!(obj.required.contains("Err"));
        assert_eq!(obj.properties["Err"], schema_for::<String>());
    }
}
