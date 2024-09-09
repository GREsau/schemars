use crate::prelude::*;
use std::ffi::{CStr, CString, OsStr, OsString};
use std::marker::PhantomData;
use std::ops::{Bound, Range, RangeInclusive};
use std::time::{Duration, SystemTime};

#[test]
fn option() {
    test!(Option<bool>)
        .assert_allows_ser_roundtrip([Some(true), None])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn result() {
    test!(Result<bool, String>)
        .assert_allows_ser_roundtrip([Ok(true), Err("oh no!".to_owned())])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn time() {
    test!(SystemTime)
        .assert_allows_ser_roundtrip([SystemTime::UNIX_EPOCH, SystemTime::now()])
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(Duration)
        .assert_allows_ser_roundtrip([Duration::from_secs(0), Duration::from_secs_f64(123.456)])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn c_strings() {
    let strings = [
        CString::default(),
        CString::new("test").unwrap(),
        CString::new([255]).unwrap(),
    ];

    test!(CString)
        .assert_allows_ser_roundtrip(strings.clone())
        .assert_matches_de_roundtrip(arbitrary_values_except(
            |v| v.as_str().is_some_and(|s| s.contains('\0')),
            "CString cannot contain null bytes, but schema does not enforce this",
        ));

    test!(Box<CStr>)
        .assert_identical::<CString>()
        .assert_allows_ser_roundtrip(strings.into_iter().map(Into::into))
        .assert_matches_de_roundtrip(arbitrary_values_except(
            |v| v.as_str().is_some_and(|s| s.contains('\0')),
            "CString cannot contain null bytes, but schema does not enforce this",
        ));
}

#[test]
fn os_strings() {
    let strings = [OsString::new(), OsString::from("test")];

    test!(OsString)
        .assert_allows_ser_roundtrip(strings.clone())
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(Box<OsStr>)
        .assert_identical::<OsString>()
        .assert_allows_ser_roundtrip(strings.into_iter().map(Into::into))
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn bound() {
    test!(Bound<i32>).assert_allows_ser_roundtrip([
        Bound::Included(123),
        Bound::Excluded(456),
        Bound::Unbounded,
    ]);
}

#[test]
fn ranges() {
    test!(Range<i32>)
        .assert_allows_ser_roundtrip([0..0, 123..456])
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(RangeInclusive<i32>)
        .assert_allows_ser_roundtrip([0..=0, 123..=456])
        .assert_matches_de_roundtrip(arbitrary_values());
}

#[test]
fn phantom_data() {
    struct DoesNotImplementJsonSchema;

    test!(PhantomData<DoesNotImplementJsonSchema>)
        .assert_allows_ser_roundtrip_default()
        .assert_matches_de_roundtrip(arbitrary_values());
}
