use crate::generator::SchemaGenerator;
use crate::schema::*;
use serde_json::json;
use std::collections::BTreeMap as Map;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SchemaTypeId(&'static str);

pub trait MakeSchema {
    fn schema_type_id() -> SchemaTypeId {
        // FIXME schema name might not be unique!
        SchemaTypeId(core::any::type_name::<Self>())
    }

    fn schema_name() -> String {
        // TODO this requires nightly
        // It's probably worth removing the default implemenation,
        //  then make every impl in this file set an explicit name
        // Or maybe hide it under feature flag?
        core::any::type_name::<Self>().replace(|c: char| !c.is_ascii_alphanumeric(), "_")
    }

    fn generates_ref_schema() -> bool {
        true
    }

    fn make_schema(gen: &mut SchemaGenerator) -> Schema;
}

macro_rules! no_ref_schema {
    () => {
        fn generates_ref_schema() -> bool {
            false
        }
    };
}

// TODO structs, enums, tuples

// TODO any other serde types other than serde_json value?
// https://github.com/serde-rs/serde/blob/ce75418e40a593fc5c0902cbf4a45305a4178dd7/serde/src/ser/impls.rs
// Cell<T>, RefCell<T>, Mutex<T>, RwLock<T>, Result<R,E>?, Duration, SystemTime,
// IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, SocketAddrV6,
// Path, PathBuf, OsStr, OsString, Wrapping<T>, Reverse<T>, AtomicBool, AtomixI8 etc.,
// NonZeroU8 etc., ArcWeak, RcWeak, BTreeMap, HashMap, (!)?, Bound?, Range?, RangeInclusive?,
// PhantomData?, CString?, CStr?, fmt::Arguments?

////////// PRIMITIVES //////////

macro_rules! simple_impl {
    ($type:tt => $instance_type:tt) => {
        impl MakeSchema for $type {
            no_ref_schema!();

            fn make_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    ..Default::default()
                }
                .into()
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

impl MakeSchema for char {
    no_ref_schema!();

    fn make_schema(_: &mut SchemaGenerator) -> Schema {
        let mut extensions = Map::new();
        extensions.insert("minLength".to_owned(), json!(1));
        extensions.insert("maxLength".to_owned(), json!(1));
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            extensions,
            ..Default::default()
        }
        .into()
    }
}

////////// ARRAYS //////////

// Does not require T: MakeSchema.
impl<T> MakeSchema for [T; 0] {
    no_ref_schema!();

    fn make_schema(_: &mut SchemaGenerator) -> Schema {
        let mut extensions = Map::new();
        extensions.insert("maxItems".to_owned(), json!(0));
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            extensions,
            ..Default::default()
        }
        .into()
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: MakeSchema> MakeSchema for [T; $len] {
                no_ref_schema!();

                fn make_schema(gen: &mut SchemaGenerator) -> Schema {
                    let mut extensions = Map::new();
                    extensions.insert("minItems".to_owned(), json!($len));
                    extensions.insert("maxItems".to_owned(), json!($len));
                    SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        items: Some(Box::from(gen.subschema_for::<T>())),
                        extensions,
                        ..Default::default()
                    }.into()
                }
            }
        )+
    }
}

array_impls! {
    01 02 03 04 05 06 07 08 09 10
    11 12 13 14 15 16 17 18 19 20
    21 22 23 24 25 26 27 28 29 30
    31 32
}

////////// SEQUENCES /////////

macro_rules! seq_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: MakeSchema,
        {
            no_ref_schema!();

            fn make_schema(gen: &mut SchemaGenerator) -> Schema
            {
                SchemaObject {
                    instance_type: Some(InstanceType::Array.into()),
                    items: Some(Box::from(gen.subschema_for::<T>())),
                    ..Default::default()
                }.into()
            }
        }
    };
}

seq_impl!(<T: Ord> MakeSchema for std::collections::BinaryHeap<T>);
seq_impl!(<T: Ord> MakeSchema for std::collections::BTreeSet<T>);
seq_impl!(<T: Eq + core::hash::Hash, H: core::hash::BuildHasher> MakeSchema for std::collections::HashSet<T, H>);
seq_impl!(<T> MakeSchema for std::collections::LinkedList<T>);
seq_impl!(<T> MakeSchema for Vec<T>);
seq_impl!(<T> MakeSchema for std::collections::VecDeque<T>);

////////// MAPS /////////

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            K: Into<String>,
            T: MakeSchema,
        {
            no_ref_schema!();

            fn make_schema(gen: &mut SchemaGenerator) -> Schema
            {
                let mut extensions = Map::new();
                extensions.insert(
                    "additionalProperties".to_owned(),
                    json!(gen.subschema_for::<T>())
                );
                SchemaObject {
                    instance_type: Some(InstanceType::Object.into()),
                    extensions,
                    ..Default::default()
                }.into()
            }
        }
    };
}

map_impl!(<K, T: Ord> MakeSchema for std::collections::BTreeMap<K, T>);
map_impl!(<K, T: Eq + core::hash::Hash, H: core::hash::BuildHasher> MakeSchema for std::collections::HashMap<K, T, H>);

////////// OPTION //////////

impl<T: MakeSchema> MakeSchema for Option<T> {
    no_ref_schema!();

    fn make_schema(gen: &mut SchemaGenerator) -> Schema {
        match gen.subschema_for::<T>() {
            Schema::Bool(true) => true.into(),
            Schema::Bool(false) => <()>::make_schema(gen),
            schema => SchemaObject {
                any_of: Some(vec![schema, <()>::make_schema(gen)]),
                ..Default::default()
            }
            .into(),
        }
    }
}

////////// DEREF //////////

macro_rules! deref_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            T: MakeSchema,
        {
            no_ref_schema!();

            fn make_schema(gen: &mut SchemaGenerator) -> Schema {
                gen.subschema_for::<T>()
            }
        }
    };
}

deref_impl!(<'a, T> MakeSchema for &'a T);
deref_impl!(<'a, T> MakeSchema for &'a mut T);
deref_impl!(<T> MakeSchema for Box<T>);
deref_impl!(<T> MakeSchema for std::rc::Rc<T>);
deref_impl!(<T> MakeSchema for std::sync::Arc<T>);
deref_impl!(<'a, T: ToOwned> MakeSchema for std::borrow::Cow<'a, T>);

////////// SERDE_JSON //////////

impl MakeSchema for serde_json::Value {
    no_ref_schema!();

    fn make_schema(_: &mut SchemaGenerator) -> Schema {
        true.into()
    }
}
