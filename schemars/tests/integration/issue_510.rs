// Test for issue #510: `schema_for!` causes a stack overflow when using a recursively defined key in a `BTreeMap`.
#[derive(schemars_derive::JsonSchema, Ord, PartialOrd, Eq, PartialEq)]
struct RecursiveBTreeMapKey(std::collections::BTreeMap<RecursiveBTreeMapKey, u32>);

#[test]
fn test_issue_510_recursive_btreemap_key_stack_overflow() {
    /* This test documents the stack overflow behavior that occurs when trying to generate
    a schema for a type with a recursively defined `BTreeMap` key.
    The issue is that JSON Schema requires all object keys to be strings, but
    `schemars` attempts to recursively process the non-string key type, leading to infinite recursion.
    
    Note: This test will cause a stack overflow, which is the bug being documented.
    In a real scenario, this should be handled more gracefully by `schemars`. */
    let gen = schemars::generate::SchemaGenerator::default();
    let _ = gen.into_root_schema_for::<RecursiveBTreeMapKey>();
}