use crate::generator::SchemaGenerator;
use crate::schema::*;
use serde_json::json;
use std::collections::BTreeMap as Map;

pub trait MakeSchema {
    fn schema_name() -> String {
        core::any::type_name::<Self>().to_owned()
    }

    fn generates_ref_schema() -> bool {
        false
    }

    fn make_schema(generator: &mut SchemaGenerator) -> Schema;
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
    fn make_schema(_: &mut SchemaGenerator) -> Schema {
        let mut extra_properties = Map::new();
        extra_properties.insert("minLength".to_owned(), json!(1));
        extra_properties.insert("maxLength".to_owned(), json!(1));
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            extra_properties,
            ..Default::default()
        }
        .into()
    }
}

////////// ARRAYS //////////

// Does not require T: MakeSchema.
impl<T> MakeSchema for [T; 0] {
    fn make_schema(_: &mut SchemaGenerator) -> Schema {
        let mut extra_properties = Map::new();
        extra_properties.insert("maxItems".to_owned(), json!(0));
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            extra_properties,
            ..Default::default()
        }
        .into()
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T: MakeSchema + 'static> MakeSchema for [T; $len]
            {
                fn make_schema(gen: &mut SchemaGenerator) -> Schema {
                    let mut extra_properties = Map::new();
                    extra_properties.insert("minItems".to_owned(), json!($len));
                    extra_properties.insert("maxItems".to_owned(), json!($len));
                    SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        items: Some(Box::from(gen.subschema_for::<T>())),
                        extra_properties,
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
            T: MakeSchema + 'static,
        {
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
            T: MakeSchema + 'static,
        {
            fn make_schema(gen: &mut SchemaGenerator) -> Schema
            {
                let mut extra_properties = Map::new();
                extra_properties.insert(
                    "additionalProperties".to_owned(),
                    json!(gen.subschema_for::<T>())
                );
                SchemaObject {
                    instance_type: Some(InstanceType::Object.into()),
                    extra_properties,
                    ..Default::default()
                }.into()
            }
        }
    };
}

map_impl!(<K, T: Ord> MakeSchema for std::collections::BTreeMap<K, T>);
map_impl!(<K, T: Eq + core::hash::Hash, H: core::hash::BuildHasher> MakeSchema for std::collections::HashMap<K, T, H>);

////////// OPTION //////////

impl<T: MakeSchema + 'static> MakeSchema for Option<T> {
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
            T: MakeSchema + 'static,
        {
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
    fn make_schema(_: &mut SchemaGenerator) -> Schema {
        true.into()
    }
}
