use crate::gen::{BoolSchemas, SchemaGenerator};
use crate::schema::*;
use crate::{JsonSchema, Map, Result};
use serde_json::json;

// TODO any other serde/json types other than serde_json value?
// TODO serde yaml value/map under feature flag
// TODO add some inline attributes
// https://github.com/serde-rs/serde/blob/ce75418e40a593fc5c0902cbf4a45305a4178dd7/serde/src/ser/impls.rs
// Cell<T>, RefCell<T>, Mutex<T>, RwLock<T>, Result<R,E>?, Duration, SystemTime,
// IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, SocketAddrV6,
// Path, PathBuf, OsStr, OsString, Wrapping<T>, Reverse<T>, AtomicBool, AtomixI8 etc.,
// NonZeroU8 etc., ArcWeak, RcWeak, BTreeMap, HashMap, (!)?, Bound?, Range?, RangeInclusive?,
// PhantomData?, CString?, CStr?, fmt::Arguments?

macro_rules! no_ref_schema {
    () => {
        fn is_referenceable() -> bool {
            false
        }
    };
}

////////// PRIMITIVES //////////

macro_rules! simple_impl {
    ($type:tt => $instance_type:ident) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                stringify!($instance_type).to_owned()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Result {
                Ok(SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    ..Default::default()
                }
                .into())
            }
        }
    };
}

simple_impl!(str => String);
simple_impl!(String => String);
simple_impl!(bool => Boolean);
simple_impl!(f32 => Number);
simple_impl!(f64 => Number);
simple_impl!(i8 => Integer);
simple_impl!(i16 => Integer);
simple_impl!(i32 => Integer);
simple_impl!(i64 => Integer);
simple_impl!(i128 => Integer);
simple_impl!(isize => Integer);
simple_impl!(u8 => Integer);
simple_impl!(u16 => Integer);
simple_impl!(u32 => Integer);
simple_impl!(u64 => Integer);
simple_impl!(u128 => Integer);
simple_impl!(usize => Integer);
simple_impl!(() => Null);

impl JsonSchema for char {
    no_ref_schema!();

    fn schema_name() -> String {
        "Character".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Result {
        let mut extensions = Map::new();
        extensions.insert("minLength".to_owned(), json!(1));
        extensions.insert("maxLength".to_owned(), json!(1));
        Ok(SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            extensions,
            ..Default::default()
        }
        .into())
    }
}

////////// ARRAYS //////////

// Does not require T: JsonSchema.
impl<T> JsonSchema for [T; 0] {
    no_ref_schema!();

    fn schema_name() -> String {
        "Empty_Array".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Result {
        let mut extensions = Map::new();
        extensions.insert("maxItems".to_owned(), json!(0));
        Ok(SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            extensions,
            ..Default::default()
        }
        .into())
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: JsonSchema> JsonSchema for [T; $len] {
                no_ref_schema!();

                fn schema_name() -> String {
                    format!("Array_Size_{}_Of_{}", $len, T::schema_name())
                }

                fn json_schema(gen: &mut SchemaGenerator) -> Result {
                    let mut extensions = Map::new();
                    extensions.insert("minItems".to_owned(), json!($len));
                    extensions.insert("maxItems".to_owned(), json!($len));
                    Ok(SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        items: Some(gen.subschema_for::<T>()?.into()),
                        extensions,
                        ..Default::default()
                    }.into())
                }
            }
        )+
    }
}

array_impls! {
     1  2  3  4  5  6  7  8  9 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}

////////// TUPLES //////////

macro_rules! tuple_impls {
    ($($len:expr => ($($name:ident)+))+) => {
        $(
            impl<$($name: JsonSchema),+> JsonSchema for ($($name,)+) {
                no_ref_schema!();

                fn schema_name() -> String {
                    ["Tuple_Of".to_owned()$(, $name::schema_name())+].join("_And_")
                }

                fn json_schema(gen: &mut SchemaGenerator) -> Result {
                    let mut extensions = Map::new();
                    extensions.insert("minItems".to_owned(), json!($len));
                    extensions.insert("maxItems".to_owned(), json!($len));
                    let items = vec![
                        $(gen.subschema_for::<$name>()?),+
                    ];
                    Ok(SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        items: Some(items.into()),
                        extensions,
                        ..Default::default()
                    }.into())
                }
            }
        )+
    }
}

tuple_impls! {
    1 => (T0)
    2 => (T0 T1)
    3 => (T0 T1 T2)
    4 => (T0 T1 T2 T3)
    5 => (T0 T1 T2 T3 T4)
    6 => (T0 T1 T2 T3 T4 T5)
    7 => (T0 T1 T2 T3 T4 T5 T6)
    8 => (T0 T1 T2 T3 T4 T5 T6 T7)
    9 => (T0 T1 T2 T3 T4 T5 T6 T7 T8)
    10 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9)
    11 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10)
    12 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11)
    13 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    14 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    15 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    16 => (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
}

////////// SEQUENCES /////////

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Array_Of_{}", T::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Result {
                Ok(SchemaObject {
                    instance_type: Some(InstanceType::Array.into()),
                    items: Some(gen.subschema_for::<T>()?.into()),
                    ..Default::default()
                }.into())
            }
        }
    };
}

seq_impl!(<T: Ord> JsonSchema for std::collections::BinaryHeap<T>);
seq_impl!(<T: Ord> JsonSchema for std::collections::BTreeSet<T>);
seq_impl!(<T: Eq + core::hash::Hash, H: core::hash::BuildHasher> JsonSchema for std::collections::HashSet<T, H>);
seq_impl!(<T> JsonSchema for std::collections::LinkedList<T>);
seq_impl!(<T> JsonSchema for Vec<T>);
seq_impl!(<T> JsonSchema for std::collections::VecDeque<T>);

////////// MAPS /////////

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            K: Into<String>,
            V: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Map_Of_{}", V::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Result {
                let subschema = gen.subschema_for::<V>()?;
                let json_schema_bool = gen.settings().bool_schemas == BoolSchemas::AdditionalPropertiesOnly
                    && subschema == gen.schema_for_any();
                let mut extensions = Map::new();
                extensions.insert(
                    "additionalProperties".to_owned(),
                    if json_schema_bool {
                        json!(true)
                    } else {
                        json!(subschema)
                    }
                );
                Ok(SchemaObject {
                    instance_type: Some(InstanceType::Object.into()),
                    extensions,
                    ..Default::default()
                }.into())
            }
        }
    };
}

map_impl!(<K: Ord, V> JsonSchema for std::collections::BTreeMap<K, V>);
map_impl!(<K: Eq + core::hash::Hash, V, H: core::hash::BuildHasher> JsonSchema for std::collections::HashMap<K, V, H>);

////////// OPTION //////////

// TODO should a field with a default set also be considered nullable?

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
                schema => SchemaObject {
                    any_of: Some(vec![schema, <()>::json_schema(gen)?]),
                    ..Default::default()
                }
                .into(),
            }
        }
        if gen.settings().option_nullable {
            let mut deref = gen.get_schema_object(&schema)?;
            deref.extensions.insert("nullable".to_owned(), json!(true));
            schema = Schema::Object(deref);
        };
        Ok(schema)
    }
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

////////// DEREF //////////

macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: ?Sized + JsonSchema,
        {
            fn is_referenceable() -> bool {
                T::is_referenceable()
            }

            fn schema_name() -> String {
                T::schema_name()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Result {
                T::json_schema(gen)
            }
        }
    };
}

deref_impl!(<'a, T> JsonSchema for &'a T);
deref_impl!(<'a, T> JsonSchema for &'a mut T);
deref_impl!(<T> JsonSchema for Box<T>);
deref_impl!(<T> JsonSchema for std::rc::Rc<T>);
deref_impl!(<T> JsonSchema for std::sync::Arc<T>);
deref_impl!(<'a, T: ToOwned> JsonSchema for std::borrow::Cow<'a, T>);

////////// SERDE_JSON //////////

impl JsonSchema for serde_json::Value {
    no_ref_schema!();

    fn schema_name() -> String {
        "Any_Value".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Result {
        Ok(gen.schema_for_any())
    }
}
