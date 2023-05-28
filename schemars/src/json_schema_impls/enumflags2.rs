use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use enumflags2::{BitFlags, _internal::RawBitFlags};

impl<T> JsonSchema for BitFlags<T> where T: JsonSchema + RawBitFlags {
    fn is_referenceable() -> bool {
        true
    }

    fn schema_name() -> String {
        format!("BitFlags_{}", T::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let target = gen.subschema_for::<T>();
        let repr = u64::json_schema(gen);
        match (repr, target) {
            (Schema::Object(mut o), Schema::Object(target)) => {
                o.metadata = target.metadata;
                Schema::Object(o).into()
            },
            (repr, _) => repr,
        }
    }
}
