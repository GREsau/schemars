use serde_json::map::Entry;
use serde_json::Value;

use crate::schema::*;

impl Schema {
    /// This function is only public for use by schemars_derive.
    ///
    /// It should not be considered part of the public API.
    #[doc(hidden)]
    pub fn flatten(mut self, other: Self) -> Schema {
        // This special null-type-schema handling is here for backward-compatibility, but needs reviewing.
        // I think it's only needed to make internally-tagged enum unit variants behave correctly, but that
        // should be handled entirely within schemars_derive.
        if other
            .as_object()
            .and_then(|o| o.get("type"))
            .and_then(|t| t.as_str())
            == Some("null")
        {
            return self;
        }

        if let Value::Object(mut obj2) = other.to_value() {
            let obj1 = self.ensure_object();

            let ap2 = obj2.remove("additionalProperties");
            if let Entry::Occupied(mut ap1) = obj1.entry("additionalProperties") {
                match ap2 {
                    Some(ap2) => {
                        flatten_additional_properties(ap1.get_mut(), ap2);
                    }
                    None => {
                        ap1.remove();
                    }
                }
            }

            for (key, value2) in obj2 {
                match obj1.entry(key) {
                    Entry::Vacant(vacant) => {
                        vacant.insert(value2);
                    }
                    Entry::Occupied(mut occupied) => {
                        match occupied.key().as_str() {
                            // This special "type" handling can probably be removed once the enum variant `with`/`schema_with` behaviour is fixed
                            "type" => match (occupied.get_mut(), value2) {
                                (Value::Array(a1), Value::Array(mut a2)) => {
                                    a2.retain(|v2| !a1.contains(v2));
                                    a1.extend(a2);
                                }
                                (v1, Value::Array(mut a2)) => {
                                    if !a2.contains(v1) {
                                        a2.push(std::mem::take(v1));
                                        *occupied.get_mut() = Value::Array(a2);
                                    }
                                }
                                (Value::Array(a1), v2) => {
                                    if !a1.contains(&v2) {
                                        a1.push(v2);
                                    }
                                }
                                (v1, v2) => {
                                    if v1 != &v2 {
                                        *occupied.get_mut() =
                                            Value::Array(vec![std::mem::take(v1), v2]);
                                    }
                                }
                            },
                            "required" => {
                                if let Value::Array(a1) = occupied.into_mut() {
                                    if let Value::Array(a2) = value2 {
                                        a1.extend(a2);
                                    }
                                }
                            }
                            "properties" | "patternProperties" => {
                                if let Value::Object(o1) = occupied.into_mut() {
                                    if let Value::Object(o2) = value2 {
                                        o1.extend(o2);
                                    }
                                }
                            }
                            _ => {
                                // leave the original value as it is (don't modify `self`)
                            }
                        };
                    }
                }
            }
        }

        self
    }
}

// TODO validate behaviour when flattening a normal struct into a struct with deny_unknown_fields
fn flatten_additional_properties(v1: &mut Value, v2: Value) {
    match (v1, v2) {
        (v1, Value::Bool(true)) => {
            *v1 = Value::Bool(true);
        }
        (v1 @ Value::Bool(false), v2) => {
            *v1 = v2;
        }
        (Value::Object(o1), Value::Object(o2)) => {
            o1.extend(o2);
        }
        _ => {}
    }
}
