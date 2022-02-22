use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::{Path, PathBuf};

macro_rules! simple_impl {
    ($type:ty => $instance_type:ident, $format:literal) => {
        simple_impl!($type => $instance_type, Some($format.to_owned()));
    };
    ($type:ty => $instance_type:ident, $format:expr) => {
        impl JsonSchema for $type {
            no_ref_schema!();

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

simple_impl!(str => String, None);
simple_impl!(bool => Boolean, None);
simple_impl!(f32 => Number, "float");
simple_impl!(f64 => Number, "double");
simple_impl!(i8 => Integer, "int8");
simple_impl!(i16 => Integer, "int16");
simple_impl!(i32 => Integer, "int32");
simple_impl!(i64 => Integer, "int64");
simple_impl!(i128 => Integer, "int128");
simple_impl!(isize => Integer, "int");
simple_impl!(() => Null, None);

simple_impl!(Ipv4Addr => String, "ipv4");
simple_impl!(Ipv6Addr => String, "ipv6");
simple_impl!(IpAddr => String, "ip");

forward_impl!(String => str);
forward_impl!(Path => str);
forward_impl!(PathBuf => str);
forward_impl!(SocketAddr => str);
forward_impl!(SocketAddrV4 => str);
forward_impl!(SocketAddrV6 => str);

macro_rules! unsigned_impl {
    ($type:ty => $instance_type:ident, $format:expr) => {
        impl JsonSchema for $type {
            no_ref_schema!();

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

unsigned_impl!(u8 => Integer, "uint8");
unsigned_impl!(u16 => Integer, "uint16");
unsigned_impl!(u32 => Integer, "uint32");
unsigned_impl!(u64 => Integer, "uint64");
unsigned_impl!(u128 => Integer, "uint128");
unsigned_impl!(usize => Integer, "uint");

impl JsonSchema for char {
    no_ref_schema!();

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
