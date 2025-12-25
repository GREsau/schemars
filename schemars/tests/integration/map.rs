use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};

#[derive(JsonSchema, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord)]
#[schemars(extend("pattern" = "^[0-9a-f]*$"))]
struct HexNumber(String);

#[derive(JsonSchema, Deserialize, Serialize, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct IndirectU32(u32);

#[derive(JsonSchema, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Enum {
    Unit1,
    Unit2,
    #[serde(untagged)]
    UntaggedI8(i8),
    #[serde(untagged)]
    UntaggedIndirectU32(IndirectU32),
}

#[derive(JsonSchema, Deserialize, Serialize, Default)]
struct Maps {
    s_map: HashMap<String, bool>,
    i_map: BTreeMap<i8, bool>,
    u_map: HashMap<u64, bool>,
    pattern_map: BTreeMap<HexNumber, bool>,
    enum_map: HashMap<Enum, bool>,
}

#[test]
fn maps() {
    test!(Maps)
        .assert_snapshot()
        .assert_allows_ser_roundtrip([Maps {
            s_map: HashMap::from_iter([("test".to_owned(), true)]),
            i_map: BTreeMap::from_iter([(-123, true), (123, true)]),
            u_map: HashMap::from_iter([(123, true)]),
            pattern_map: BTreeMap::from_iter([(HexNumber("b4df00d".to_owned()), true)]),
            enum_map: HashMap::from_iter([(Enum::Unit1, true)]),
        }])
        .assert_allows_ser_only(
            // serde allow serializing untagged non-string newtype variants, but not deserializing them.
            [
                Maps {
                    enum_map: HashMap::from_iter([(Enum::UntaggedI8(-100), true)]),
                    ..Default::default()
                },
                Maps {
                    // It's unfortunate that this adds an unused `IndirectU32` to the schema's `$defs`,
                    // but it's ultimately harmless.
                    enum_map: HashMap::from_iter([(
                        Enum::UntaggedIndirectU32(IndirectU32(1000000)),
                        true,
                    )]),
                    ..Default::default()
                },
            ],
        )
        .assert_matches_de_roundtrip(arbitrary_values());
}
