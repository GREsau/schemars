mod util;
use schemars::{gen::SchemaSettings, JsonSchema};
use util::*;

#[allow(dead_code)]
#[derive(JsonSchema)]
/**
 *
 * # This is the struct's title
 *
 * This is the struct's description.
 *
 */
struct MyStruct {
    /// # An integer
    my_int: i32,
    my_undocumented_bool: bool,
    /// A unit struct instance
    my_unit: MyUnitStruct,
}

/// # A Unit
///
#[derive(JsonSchema)]
struct MyUnitStruct;

#[allow(dead_code)]
#[doc = " # This is the enum's title "]
#[doc = " This is "]
#[derive(JsonSchema)]
#[doc = " the enum's description."]
enum MyEnum {
    UndocumentedUnit,
    UndocumentedUnit2,
    /// This comment is included in the generated schema :)
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
#[allow(dead_code)]
#[derive(JsonSchema)]
#[schemars(description = "New description")]
struct OverrideDocs {
    /// # Overridden
    #[schemars(title = "My integer", description = "This is an i32")]
    my_int: i32,
    /// # Overridden
    /// Also overridden
    #[schemars(title = "", description = "")]
    my_undocumented_bool: bool,
}

#[test]
fn doc_comments_override() -> TestResult {
    test_default_generated_schema::<OverrideDocs>("doc_comments_override")
}

/// # RawDescriptionText struct
///
/// This struct is commented using markdown.
///
/// ## Example
///
/// A code:
///
/// ```
/// let _json = serde_json::to_string(&MarkupDocs { my_int: 10 }).unwrap();
/// ```
///
/// A table:
///
/// | Col1     | Col2     |
/// |----------|----------|
/// | schemars | schemars |
/// | serde    | serde    |
#[allow(dead_code)]
#[derive(JsonSchema)]
struct RawDescriptionText {
    /// # Documented field
    ///
    /// This is first line of a description
    /// This is second line of a description
    my_int: i32,
}

#[test]
fn raw_description_text() -> TestResult {
    let settings = SchemaSettings::draft07().with(|s| s.raw_description_text = true);
    test_generated_schema::<RawDescriptionText>("raw_description_text", settings)
}
