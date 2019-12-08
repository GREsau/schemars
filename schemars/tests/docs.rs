mod util;
use schemars::JsonSchema;
use util::*;

#[derive(Debug, JsonSchema)]
/**
 *
 * # This is the struct's title
 *
 * This is the struct's description.
 *
 */
pub struct MyStruct {
    /// # An integer
    pub my_int: i32,
    pub my_undocumented_bool: bool,
    /// A unit struct instance
    pub my_unit: MyUnitStruct,
}

/// # A Unit
///
#[derive(Debug, JsonSchema)]
pub struct MyUnitStruct;

/// # This is the enum's title
/// This is
#[derive(Debug, JsonSchema)]
/// the enum's description.
pub enum MyEnum {
    UndocumentedUnit,
    /// This comment is not included in the generated schema :(
    DocumentedUnit,
    /// ## Complex variant
    /// This is a struct-like variant.
    Complex {
        /// ### A nullable string
        ///
        /// This field is a nullable string.
        ///
        /// This
        ///is
        ///   the second
        ///  line!
        ///
        ///
        ///
        ///
        /// And this is the third!
        my_nullable_string: Option<String>,
    },
}

#[test]
fn doc_comments_struct() -> TestResult {
    test_default_generated_schema::<MyStruct>("doc_comments_struct")
}

#[test]
fn doc_comments_enum() -> TestResult {
    test_default_generated_schema::<MyEnum>("doc_comments_enum")
}
