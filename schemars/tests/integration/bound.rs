use crate::prelude::*;
use std::marker::PhantomData;

struct MyIterator;

impl Iterator for MyIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

// The default trait bounds would require T to implement JsonSchema, which MyIterator does not.
// Ideally we wouldn't need the `bound` attribute here at all - it should be possible to better
// infer automatic trait bounds (tracked in https://github.com/GREsau/schemars/issues/168)
#[derive(JsonSchema, Serialize, Deserialize)]
#[schemars(bound = "T::Item: JsonSchema")]
pub struct MyContainer<T>
where
    T: Iterator,
{
    pub associated: T::Item,
    pub generic: PhantomData<T>,
}

#[test]
fn manual_bound_set() {
    test!(MyContainer<MyIterator>)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([MyContainer {
            associated: "test".to_owned(),
            generic: PhantomData,
        }])
        .assert_matches_de_roundtrip(arbitrary_values());
}
