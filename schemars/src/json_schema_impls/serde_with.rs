use serde_with::formats::Format;
use serde_with::*;

use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

macro_rules! schema_impl {
    ($ident:ident$(<$($T:ident$(:$Bound:path)?),+>)?, $name:block, $schema:block, $gen:ident) => {
        impl$(<$($T: JsonSchema $(+ $Bound)?),+>)? JsonSchema for $ident $(<$($T),+>)? {
            fn schema_name() -> String $name

            fn json_schema($gen: &mut SchemaGenerator) -> Schema $schema
        }
    };
    ($ident:ident$(<$($T:ident$(:$Bound:path)?),+>)? = $alias:ty) => {
        schema_impl!($ident$(<$($T$(:$Bound)?),+>)?, { <$alias>::schema_name() }, { <$alias>::json_schema(gen) }, gen);
    };
}

schema_impl!(As<T> = T);
schema_impl!(BoolFromInt = u8);
schema_impl!(Bytes = Vec<u8>);
schema_impl!(
    BytesOrString,
    { "BytesOrString".to_owned() },
    {
        let mut schema = SchemaObject::default();
        schema.subschemas().any_of = Some(vec![
            gen.subschema_for::<Vec<u8>>(),
            gen.subschema_for::<String>(),
        ]);
        schema.into()
    },
    gen
);
schema_impl!(DefaultOnError<T> = T);
schema_impl!(DefaultOnNull<T> = T);
schema_impl!(DisplayFromStr = String);

macro_rules! with_format {
    ($($ident:ident),*) => {
        $( schema_impl!($ident<FORMAT: Format> = FORMAT);)*
    };
}
with_format!(
    DurationMicroSeconds,
    DurationMicroSecondsWithFrac,
    DurationMilliSeconds,
    DurationMilliSecondsWithFrac,
    DurationNanoSeconds,
    DurationNanoSecondsWithFrac,
    DurationSeconds,
    DurationSecondsWithFrac
);

schema_impl!(FromInto<T> = T);
schema_impl!(Map<K, V>,
    { format!("Map_of_{}_to_{}", K::schema_name(), V::schema_name()) },
    {
        let subschema = gen.subschema_for::<V>();
        SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(Box::new(ObjectValidation {
                additional_properties: Some(Box::new(subschema)),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    },
    gen
);
schema_impl!(NoneAsEmptyString = Option<String>);
schema_impl!(
    OneOrMany<T>,
    { format!("OneOrMany_of_{}", T::schema_name()) },
    {
        let mut schema = SchemaObject::default();
        schema.subschemas().any_of = Some(vec![
            gen.subschema_for::<Vec<T>>(),
            gen.subschema_for::<T>(),
        ]);
        schema.into()
    },
    gen
);

impl<T: JsonSchema> JsonSchema for PickFirst<(T,)> {
    fn schema_name() -> String {
        T::schema_name()
    }
    fn json_schema(gen: &mut crate::gen::SchemaGenerator) -> Schema {
        T::json_schema(gen)
    }
}
macro_rules! pick_first {
    ($($Ts:ident),+, $fmt:literal) => {
        impl<$($Ts: JsonSchema),+> JsonSchema for PickFirst<($($Ts),+)> {
            fn schema_name() -> String {
                format!($fmt, $($Ts::schema_name()),+)
            }
            fn json_schema(gen: &mut crate::gen::SchemaGenerator) -> Schema {
                let mut schema = SchemaObject::default();
                schema.subschemas().any_of =
                    Some(vec![$(gen.subschema_for::<$Ts>()),+]);
                schema.into()
            }
        }
    };
}
// only implemented for 1-4
pick_first!(A, B, "{}_or_{}");
pick_first!(A, B, C, "{}_or_{}_or_{}");
pick_first!(A, B, C, D, "{}_or_{}_or_{}_or_{}");

schema_impl!(Seq<T> = Vec<T>);
impl<Sep, T: JsonSchema> JsonSchema for StringWithSeparator<Sep, T> {
    fn schema_name() -> String {
        T::schema_name()
    }
    fn json_schema(gen: &mut crate::gen::SchemaGenerator) -> Schema {
        T::json_schema(gen)
    }
}

with_format!(
    TimestampMicroSeconds,
    TimestampMicroSecondsWithFrac,
    TimestampMilliSeconds,
    TimestampMilliSecondsWithFrac,
    TimestampNanoSeconds,
    TimestampNanoSecondsWithFrac,
    TimestampSeconds,
    TimestampSecondsWithFrac
);

schema_impl!(TryFromInto<T> = T);
schema_impl!(VecSkipError<T> = Vec<T>);
