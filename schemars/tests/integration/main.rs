#![allow(clippy::disallowed_names)]

#[cfg(feature = "arrayvec07")]
mod arrayvec;
mod bound;
#[cfg(feature = "bytes1")]
mod bytes;
#[cfg(feature = "chrono04")]
mod chrono;
mod contract;
mod crate_alias;
#[cfg(any(feature = "rust_decimal1", feature = "bigdecimal04"))]
mod decimal;
mod default;
mod deprecated;
mod docs;
#[cfg(feature = "either1")]
mod either;
mod enum_repr;
mod enums;
mod enums_deny_unknown_fields;
mod enums_flattened;
mod extend;
mod flatten;
mod std_types;

mod prelude {
    pub use crate::test;
    pub use crate::test_helper::{arbitrary_values, arbitrary_values_except};
    pub use schemars::JsonSchema;
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
}

mod test_helper;

#[macro_export]
macro_rules! test {
    ($type:ty, $settings:expr) => {
        $crate::test_helper::TestHelper::<$type>::new(
            {
                fn f() {}
                fn type_name_of_val<T>(_: T) -> &'static str {
                    core::any::type_name::<T>()
                }
                let test_name = type_name_of_val(f)
                    .trim_end_matches("::f")
                    .split("::")
                    .last()
                    .unwrap();

                format!("{}~{}", core::file!(), test_name)
            },
            $settings,
        )
    };
    ($type:ty) => {
        test!($type, schemars::generate::SchemaSettings::default())
    };
}
