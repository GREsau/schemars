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
mod deref;
mod maps;
mod primitives;
mod sequences;
mod serdejson;
mod tuple;

// TODO serde yaml value/map under feature flag
// https://github.com/serde-rs/serde/blob/ce75418e40a593fc5c0902cbf4a45305a4178dd7/serde/src/ser/impls.rs
// Result<R,E>?, Duration, SystemTime,
// IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, SocketAddrV6,
// Path, PathBuf, OsStr, OsString, Wrapping<T>, Reverse<T>, AtomicBool, AtomicI8 etc.,
// NonZeroU8 etc., ArcWeak, RcWeak, (!)?, Bound?, Range?, RangeInclusive?,
// CString?, CStr?, fmt::Arguments?
