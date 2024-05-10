use serde_json::Value;

use crate::schema::*;

impl Schema {
    /// This function is only public for use by schemars_derive.
    ///
    /// It should not be considered part of the public API.
    #[doc(hidden)]
    pub fn flatten(mut self, other: Self) -> Schema {
        if let Value::Object(obj2) = other.into() {
            let obj1 = self.ensure_object();

            for (key, value2) in obj2 {
                match obj1.entry(key) {
                    serde_json::map::Entry::Vacant(vacant) => {
                        vacant.insert(value2);
                    }
                    serde_json::map::Entry::Occupied(mut occupied) => {
                        match occupied.key().as_str() {
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
                            "additionalProperties" => {
                                let value1 = std::mem::take(occupied.get_mut());
                                occupied.insert(flatten_additional_properties(value1, value2));
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

fn flatten_additional_properties(v1: Value, v2: Value) -> Value {
    match (v1, v2) {
        (Value::Bool(true), _) | (_, Value::Bool(true)) => Value::Bool(true),
        (Value::Bool(false), v) | (v, Value::Bool(false)) => v,
        (Value::Object(mut o1), Value::Object(o2)) => {
            o1.extend(o2);
            Value::Object(o1)
        }
        (v, _) => v,
    }
}
