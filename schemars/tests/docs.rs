mod util;
use schemars::{gen::SchemaSettings, JsonSchema};
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

#[doc = " # This is the enum's title "]
#[doc = " This is "]
#[derive(Debug, JsonSchema)]
#[doc = " the enum's description."]
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
fn doc_comments_struct_ref_siblings() -> TestResult {
    let settings = SchemaSettings::draft2019_09();
    test_generated_schema::<MyStruct>("doc_comments_struct_ref_siblings", settings)
}

#[test]
fn doc_comments_enum() -> TestResult {
    test_default_generated_schema::<MyEnum>("doc_comments_enum")
}

/// # OverrideDocs struct
/// This description should be overridden
#[derive(Debug, JsonSchema)]
#[schemars(description = "New description")]
pub struct OverrideDocs {
    /// # Overridden
    #[schemars(title = "My integer", description = "This is an i32")]
    pub my_int: i32,
    /// # Overridden
    /// Also overridden
    #[schemars(title = "", description = "")]
    pub my_undocumented_bool: bool,
}

#[test]
fn doc_comments_override() -> TestResult {
    test_default_generated_schema::<OverrideDocs>("doc_comments_override")
}
