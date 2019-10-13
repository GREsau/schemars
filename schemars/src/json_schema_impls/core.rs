use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::{JsonSchema, Result};
use serde_json::json;

impl<T: JsonSchema> JsonSchema for Option<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Nullable_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        let mut schema = if gen.settings().option_nullable {
            T::json_schema(gen)?
        } else {
            gen.subschema_for::<T>()?
        };
        if gen.settings().option_add_null_type {
            schema = match schema {
                Schema::Bool(true) => Schema::Bool(true),
                Schema::Bool(false) => <()>::json_schema(gen)?,
                Schema::Object(
                    obj @ SchemaObject {
                        instance_type: Some(_),
                        ..
                    },
                ) => Schema::Object(with_null_type(obj)),
                schema => SchemaObject {
                    any_of: Some(vec![schema, <()>::json_schema(gen)?]),
                    ..Default::default()
                }
                .into(),
            }
        }
        if gen.settings().option_nullable {
            let mut deref: SchemaObject = gen.dereference(schema)?.into();
            deref.extensions.insert("nullable".to_owned(), json!(true));
            schema = Schema::Object(deref);
        };
        Ok(schema)
    }

    fn json_schema_non_null(gen: &mut SchemaGenerator) -> Result {
        T::json_schema_non_null(gen)
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

impl<T: ?Sized> JsonSchema for std::marker::PhantomData<T> {
    no_ref_schema!();

    fn schema_name() -> String {
        <()>::schema_name()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        <()>::json_schema(gen)
    }
}

impl JsonSchema for std::convert::Infallible {
    no_ref_schema!();

    fn schema_name() -> String {
        "Never".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        Ok(gen.schema_for_none())
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
        assert_eq!(schema.any_of.is_none(), true);
    }

    #[test]
    fn schema_for_option_with_ref() {
        use crate as schemars;
        #[derive(JsonSchema)]
        struct Foo;

        let schema = schema_object_for::<Option<Foo>>();
        assert_eq!(schema.instance_type, None);
        assert_eq!(schema.extensions.get("nullable"), None);
        assert_eq!(schema.any_of.is_some(), true);
        let any_of = schema.any_of.unwrap();
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
        assert_eq!(schema.any_of.is_none(), true);
    }
}
