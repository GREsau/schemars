use crate::*;
use std::{borrow::Cow, format};

macro_rules! impls {
    ($($ident:ident<const V: $ty:ident>),* $(,)?) => {$(
        impl<const V: $ty> JsonSchema for monostate1::$ident<V> {
            fn schema_name() -> Cow<'static, str> {
                Cow::Owned(format!("{}<{}>", stringify!($ident), V))
            }
            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({
                    "const": V,
                })
            }
            fn inline_schema() -> bool {
                true
            }
            fn schema_id() -> Cow<'static, str> {
                Cow::Owned(format!("monostate::{}", Self::schema_name()))
            }
        }
    )*};
}

impls! {
    MustBeBool<const V: bool>,
    MustBeChar<const V: char>,

    MustBeI8<const V: i8>,
    MustBeI16<const V: i16>,
    MustBeI32<const V: i32>,
    MustBeI64<const V: i64>,
    MustBeI128<const V: i128>,

    MustBeU8<const V: u8>,
    MustBeU16<const V: u16>,
    MustBeU32<const V: u32>,
    MustBeU64<const V: u64>,
    MustBeU128<const V: u128>,
}

impl<V: monostate1::ConstStr> JsonSchema for monostate1::MustBeStr<V> {
    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!("{}<{}>", stringify!($ident), V::VALUE))
    }
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "const": V::VALUE,
        })
    }
    fn inline_schema() -> bool {
        true
    }
    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(format!("monostate::{}", Self::schema_name()))
    }
}
