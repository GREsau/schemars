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
        let t = gen.subschema_for::<T>();
        let r = u64::json_schema(gen);
        match r {
            Schema::Object(mut o) => {
               match t {
                   Schema::Object(mut to) => {
                      o.metadata = to.metadata;
                   },
                   _ => ()
               }
            },
            _ => ()
        }
        r
    }
}
