use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::borrow::Cow;

use arbitrary_int::*;

macro_rules! arbitrary_int_impl
{
    ($type:ty => $primitive:ty) => {
        impl JsonSchema for $type {
            no_ref_schema!();
        
            fn schema_name() -> String {
                stringify!($type).to_owned()
            }
        
            fn schema_id() -> Cow<'static, str> {
                Cow::Borrowed(stringify!(std::num::$type))
            }
        
            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let mut schema: SchemaObject = <$primitive>::json_schema(gen).into();
                schema.number().maximum = Some(<$type>::MAX.value() as f64);
                schema.into()
            }
        }
    }
}

arbitrary_int_impl!(u1 => u8);
arbitrary_int_impl!(u2 => u8);
arbitrary_int_impl!(u3 => u8);
arbitrary_int_impl!(u4 => u8);
arbitrary_int_impl!(u5 => u8);
arbitrary_int_impl!(u6 => u8);
arbitrary_int_impl!(u7 => u8);
arbitrary_int_impl!(u9 => u16);
arbitrary_int_impl!(u10 => u16);
arbitrary_int_impl!(u11 => u16);
arbitrary_int_impl!(u12 => u16);
arbitrary_int_impl!(u13 => u16);
arbitrary_int_impl!(u14 => u16);
arbitrary_int_impl!(u15 => u16);
arbitrary_int_impl!(u17 => u32);
arbitrary_int_impl!(u18 => u32);
arbitrary_int_impl!(u19 => u32);
arbitrary_int_impl!(u20 => u32);
arbitrary_int_impl!(u21 => u32);
arbitrary_int_impl!(u22 => u32);
arbitrary_int_impl!(u23 => u32);
arbitrary_int_impl!(u24 => u32);
arbitrary_int_impl!(u25 => u32);
arbitrary_int_impl!(u26 => u32);
arbitrary_int_impl!(u27 => u32);
arbitrary_int_impl!(u28 => u32);
arbitrary_int_impl!(u29 => u32);
arbitrary_int_impl!(u30 => u32);
arbitrary_int_impl!(u31 => u32);
arbitrary_int_impl!(u33 => u64);
arbitrary_int_impl!(u34 => u64);
arbitrary_int_impl!(u35 => u64);
arbitrary_int_impl!(u36 => u64);
arbitrary_int_impl!(u37 => u64);
arbitrary_int_impl!(u38 => u64);
arbitrary_int_impl!(u39 => u64);
arbitrary_int_impl!(u40 => u64);
arbitrary_int_impl!(u41 => u64);
arbitrary_int_impl!(u42 => u64);
arbitrary_int_impl!(u43 => u64);
arbitrary_int_impl!(u44 => u64);
arbitrary_int_impl!(u45 => u64);
arbitrary_int_impl!(u46 => u64);
arbitrary_int_impl!(u47 => u64);
arbitrary_int_impl!(u48 => u64);
arbitrary_int_impl!(u49 => u64);
arbitrary_int_impl!(u50 => u64);
arbitrary_int_impl!(u51 => u64);
arbitrary_int_impl!(u52 => u64);
arbitrary_int_impl!(u53 => u64);
arbitrary_int_impl!(u54 => u64);
arbitrary_int_impl!(u55 => u64);
arbitrary_int_impl!(u56 => u64);
arbitrary_int_impl!(u57 => u64);
arbitrary_int_impl!(u58 => u64);
arbitrary_int_impl!(u59 => u64);
arbitrary_int_impl!(u60 => u64);
arbitrary_int_impl!(u61 => u64);
arbitrary_int_impl!(u62 => u64);
arbitrary_int_impl!(u63 => u64);
arbitrary_int_impl!(u65 => u128);
arbitrary_int_impl!(u66 => u128);
arbitrary_int_impl!(u67 => u128);
arbitrary_int_impl!(u68 => u128);
arbitrary_int_impl!(u69 => u128);
arbitrary_int_impl!(u70 => u128);
arbitrary_int_impl!(u71 => u128);
arbitrary_int_impl!(u72 => u128);
arbitrary_int_impl!(u73 => u128);
arbitrary_int_impl!(u74 => u128);
arbitrary_int_impl!(u75 => u128);
arbitrary_int_impl!(u76 => u128);
arbitrary_int_impl!(u77 => u128);
arbitrary_int_impl!(u78 => u128);
arbitrary_int_impl!(u79 => u128);
arbitrary_int_impl!(u80 => u128);
arbitrary_int_impl!(u81 => u128);
arbitrary_int_impl!(u82 => u128);
arbitrary_int_impl!(u83 => u128);
arbitrary_int_impl!(u84 => u128);
arbitrary_int_impl!(u85 => u128);
arbitrary_int_impl!(u86 => u128);
arbitrary_int_impl!(u87 => u128);
arbitrary_int_impl!(u88 => u128);
arbitrary_int_impl!(u89 => u128);
arbitrary_int_impl!(u90 => u128);
arbitrary_int_impl!(u91 => u128);
arbitrary_int_impl!(u92 => u128);
arbitrary_int_impl!(u93 => u128);
arbitrary_int_impl!(u94 => u128);
arbitrary_int_impl!(u95 => u128);
arbitrary_int_impl!(u96 => u128);
arbitrary_int_impl!(u97 => u128);
arbitrary_int_impl!(u98 => u128);
arbitrary_int_impl!(u99 => u128);
arbitrary_int_impl!(u100 => u128);
arbitrary_int_impl!(u101 => u128);
arbitrary_int_impl!(u102 => u128);
arbitrary_int_impl!(u103 => u128);
arbitrary_int_impl!(u104 => u128);
arbitrary_int_impl!(u105 => u128);
arbitrary_int_impl!(u106 => u128);
arbitrary_int_impl!(u107 => u128);
arbitrary_int_impl!(u108 => u128);
arbitrary_int_impl!(u109 => u128);
arbitrary_int_impl!(u110 => u128);
arbitrary_int_impl!(u111 => u128);
arbitrary_int_impl!(u112 => u128);
arbitrary_int_impl!(u113 => u128);
arbitrary_int_impl!(u114 => u128);
arbitrary_int_impl!(u115 => u128);
arbitrary_int_impl!(u116 => u128);
arbitrary_int_impl!(u117 => u128);
arbitrary_int_impl!(u118 => u128);
arbitrary_int_impl!(u119 => u128);
arbitrary_int_impl!(u120 => u128);
arbitrary_int_impl!(u121 => u128);
arbitrary_int_impl!(u122 => u128);
arbitrary_int_impl!(u123 => u128);
arbitrary_int_impl!(u124 => u128);
arbitrary_int_impl!(u125 => u128);
arbitrary_int_impl!(u126 => u128);
arbitrary_int_impl!(u127 => u128);
