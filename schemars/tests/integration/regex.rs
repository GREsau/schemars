use crate::prelude::*;
use regex::Regex;

#[test]
fn regex_compile() {
    test!(Regex)
        .assert_snapshot()
        .assert_allows_de_roundtrip(
            [
                r"^\d+$",
                r"^[a-zA-Z_][a-zA-Z0-9_]*$",
                r"^\w+@\w+\.\w+$",
                r"(foo|bar|baz)",
                r"(?i)CaseInsensitive",
                r"^\p{Greek}+${3}",
            ]
            .into_iter()
            .map(Value::from),
        )
        .assert_rejects_de(
            [
                r"[",
                r"(*)",
                r"(?P<incomplete)",
                r"foo(bar",
                r"(?z)",           // unsupported flag
                r"(?<name>[a-z]+", // unterminated group
            ]
            .into_iter()
            .map(Value::from),
        )
        // Arbitrary strings should roundtrip if they parse
        .assert_matches_de_roundtrip(arbitrary_values());
}
