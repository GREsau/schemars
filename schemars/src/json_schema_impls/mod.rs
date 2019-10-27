macro_rules! no_ref_schema {
    () => {
        fn is_referenceable() -> bool {
            false
        }
    };
}

mod array;
#[cfg(feature = "chrono")]
mod chrono;
mod core;
mod ffi;
mod maps;
mod primitives;
mod sequences;
mod serdejson;
mod time;
mod tuple;
mod wrapper;
