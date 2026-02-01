use crate::prelude::*;
use pretty_assertions::assert_eq;
use schemars::Schema;
use std::fmt::Write;

// This test ensures that `extend` and `transform` attributes are applied after other attributes,
// and transforms are applied in the order they are defined.

#[derive(JsonSchema, Deserialize, Serialize)]
#[schemars(transform = suffix_description, description = "[Overwritten]", extend("description" = "The enum", "suffix" = "..."))]
#[serde(untagged)]
enum Untagged {
    #[schemars(transform = suffix_description, description = "The variant", extend("suffix" = "?"))]
    A {
        #[schemars(transform = suffix_description, description = "The field", extend("suffix" = "!"))]
        #[schemars(range(min = 1), transform = remove_minimum_and_default)]
        #[serde(default)]
        i: i32,
    },
}

fn suffix_description(schema: &mut Schema) {
    let minimum = schema.get("minimum").map(Value::to_string);

    let Some(Value::String(suffix)) = schema.remove("suffix") else {
        panic!("expected `suffix` to be present and a string");
    };
    let Some(Value::String(description)) = schema.get_mut("description") else {
        panic!("expected `description` to be present and a string");
    };

    description.push_str(&suffix);

    if let Some(minimum) = minimum {
        write!(description, " (At least {})", minimum).unwrap();
    }
}

fn remove_minimum_and_default(schema: &mut Schema) {
    schema.remove("minimum");
    schema.remove("default");
}

#[test]
fn attributes_applied_in_order() {
    test!(Untagged).assert_snapshot().custom(|schema, _| {
        assert_eq!(schema.pointer("/description"), Some(&json!("The enum...")));
        assert_eq!(
            schema.pointer("/anyOf/0/description"),
            Some(&json!("The variant?"))
        );
        assert_eq!(
            schema.pointer("/anyOf/0/properties/i/description"),
            Some(&json!("The field! (At least 1)"))
        );
        assert_eq!(schema.pointer("/anyOf/0/properties/i/default"), None);
        assert_eq!(schema.pointer("/anyOf/0/properties/i/minimum"), None);
    });
}
