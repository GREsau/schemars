use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use num_traits::bounds::Bounded;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::{Path, PathBuf};

// Implementation for numbers whose maximum is small enough to be expressed exactly as an f64.
macro_rules! small_number_impl {
    ($type:ty => $instance_type:ident, $format:expr) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                $format.to_owned()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                let mut schema = SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    format: Some($format.to_owned()),
                    ..Default::default()
                };
                schema.number().minimum = Some(<$type as Bounded>::min_value() as f64);
                schema.number().maximum = Some(<$type as Bounded>::max_value() as f64);
                schema.into()
            }
        }
    };
}
small_number_impl!(i8 => Integer, "int8");
small_number_impl!(i16 => Integer, "int16");
small_number_impl!(i32 => Integer, "int32");
small_number_impl!(u8 => Integer, "uint8");
small_number_impl!(u16 => Integer, "uint16");
small_number_impl!(u32 => Integer, "uint32");

// Implementation for unsigned numbers too large to be handled by small_number_impl: we can at
// least set the minimum value to zero.
macro_rules! unsigned_impl {
    ($type:ty => $instance_type:ident, $format:expr) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                $format.to_owned()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                let mut schema = SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    format: Some($format.to_owned()),
                    ..Default::default()
                };
                schema.number().minimum = Some(0.0);
                schema.into()
            }
        }
    };
}

unsigned_impl!(u64 => Integer, "uint64");
unsigned_impl!(u128 => Integer, "uint128");
unsigned_impl!(usize => Integer, "uint");

macro_rules! simple_impl {
    ($type:ty => $instance_type:ident) => {
        simple_impl!($type => $instance_type, stringify!($instance_type), None);
    };
    ($type:ty => $instance_type:ident, $format:literal) => {
        simple_impl!($type => $instance_type, $format, Some($format.to_owned()));
    };
    ($type:ty => $instance_type:ident, $name:expr, $format:expr) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                $name.to_owned()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    format: $format,
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
simple_impl!(f32 => Number, "float");
simple_impl!(f64 => Number, "double");
simple_impl!(i64 => Integer, "int64");
simple_impl!(i128 => Integer, "int128");
simple_impl!(isize => Integer, "int");
simple_impl!(() => Null);

simple_impl!(Path => String);
simple_impl!(PathBuf => String);

simple_impl!(Ipv4Addr => String, "ipv4");
simple_impl!(Ipv6Addr => String, "ipv6");
simple_impl!(IpAddr => String, "ip");

simple_impl!(SocketAddr => String);
simple_impl!(SocketAddrV4 => String);
simple_impl!(SocketAddrV6 => String);

impl JsonSchema for char {
    no_ref_schema!();

    fn schema_name() -> String {
        "Character".to_owned()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            string: Some(Box::new(StringValidation {
                min_length: Some(1),
                max_length: Some(1),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
