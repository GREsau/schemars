use crate::prelude::*;
use std::{borrow::Cow, marker::PhantomData};

struct MyIterator;

impl Iterator for MyIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

#[derive(JsonSchema, Serialize, Deserialize)]
pub struct SomeGeneric<T>(PhantomData<T>);

impl<T> Default for SomeGeneric<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

// The default trait bounds would require T to implement JsonSchema, which MyIterator does not.
#[derive(JsonSchema, Serialize, Deserialize)]
#[serde(bound = "T::Item : Serialize + serde::de::DeserializeOwned")]
pub struct MyContainer<T>
where
    T: IntoIterator,
{
    pub associated: T::Item,
    pub generic1: SomeGeneric<T>,
    pub generic2: SomeGeneric<T>,
    pub phantom: PhantomData<T>,
}

#[test]
fn automatic_bound() {
    test!(MyContainer<MyIterator>)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([MyContainer {
            associated: "test".to_owned(),
            generic1: Default::default(),
            generic2: Default::default(),
            phantom: PhantomData,
        }])
        .assert_matches_de_roundtrip(arbitrary_values());

    test!(MyContainer<&[String]>).assert_identical::<MyContainer<Vec<&str>>>();
    assert_ne!(
        <MyContainer<&[String]>>::schema_id(),
        <MyContainer<&[i32]>>::schema_id()
    );
}

#[derive(JsonSchema, Serialize)]
struct S<'a, 'b, T: 'a + 'b + ?Sized + ToOwned> {
    a: &'a T,
    b: &'b T,
    aa: Option<&'a &'a T>,
    ab: Option<&'a &'b T>,
    ba: Option<&'b &'a T>,
    bb: Option<&'b &'b T>,
    a_cow: Cow<'a, T>,
    b_cow: Cow<'b, T>,
}

#[test]
fn automatic_bound_lifetimes() {
    test!(S<str>).assert_snapshot().assert_allows_ser_only([S {
        a: "a",
        b: "b",
        aa: None,
        ab: None,
        ba: None,
        bb: None,
        a_cow: Cow::Borrowed("a"),
        b_cow: Cow::Owned("b".to_owned()),
    }]);
}
