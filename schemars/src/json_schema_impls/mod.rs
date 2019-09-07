macro_rules! no_ref_schema {
    () => {
        fn is_referenceable() -> bool {
            false
        }
    };
}

mod array;
mod core;
mod deref;
mod maps;
mod primitives;
mod sequences;
mod serdejson;
mod tuple;

// TODO chrono types under feature flag
// TODO serde yaml value/map under feature flag
// https://github.com/serde-rs/serde/blob/ce75418e40a593fc5c0902cbf4a45305a4178dd7/serde/src/ser/impls.rs
// Cell<T>, RefCell<T>, Mutex<T>, RwLock<T>, Result<R,E>?, Duration, SystemTime,
// IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6, SocketAddrV6,
// Path, PathBuf, OsStr, OsString, Wrapping<T>, Reverse<T>, AtomicBool, AtomixI8 etc.,
// NonZeroU8 etc., ArcWeak, RcWeak, (!)?, Bound?, Range?, RangeInclusive?,
// CString?, CStr?, fmt::Arguments?
