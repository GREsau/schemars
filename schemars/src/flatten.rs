use serde_json::map::Entry;
use serde_json::Value;

use crate::Schema;

impl Schema {
    /// This function is only public for use by schemars_derive.
    ///
    /// It should not be considered part of the public API.
    #[doc(hidden)]
    pub fn flatten(mut self, other: Self) -> Schema {
        if let Value::Object(obj2) = other.to_value() {
            let obj1 = self.ensure_object();

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
