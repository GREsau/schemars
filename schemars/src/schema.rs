/*!
JSON Schema types.
*/

use ref_cast::{ref_cast_custom, RefCastCustom};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A JSON Schema.
#[derive(Debug, Clone, PartialEq, RefCastCustom)]
#[repr(transparent)]
pub struct Schema(Value);

impl<'de> Deserialize<'de> for Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Schema::validate(&value)?;
        Ok(Schema(value))
    }
}

impl Serialize for Schema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser::OrderedKeywordWrapper(&self.0).serialize(serializer)
    }
}

impl Schema {
    pub fn new() -> Self {
        Self(Value::Object(Map::new()))
    }

    pub fn new_ref(reference: String) -> Self {
        let mut map = Map::new();
        map.insert("$ref".to_owned(), Value::String(reference));
        Self(Value::Object(map))
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.0.as_bool()
    }

    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        self.0.as_object()
    }

    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        self.0.as_object_mut()
    }

    pub(crate) fn try_to_object(self) -> Result<Map<String, Value>, bool> {
        match self.0 {
            Value::Object(m) => Ok(m),
            Value::Bool(b) => Err(b),
            _ => unreachable!(),
        }
    }

    pub fn to_value(self) -> Value {
        self.0
    }

    pub fn ensure_object(&mut self) -> &mut Map<String, Value> {
        if let Some(b) = self.as_bool() {
            let mut map = Map::new();
            if !b {
                map.insert("not".into(), Value::Object(Map::new()));
            }
            self.0 = Value::Object(map);
        }

        self.as_object_mut()
            .expect("Schema value should be of type Object.")
    }

    pub(crate) fn has_type(&self, ty: &str) -> bool {
        match self.0.get("type") {
            Some(Value::Array(values)) => values.iter().any(|v| v.as_str() == Some(ty)),
            Some(Value::String(s)) => s == ty,
            _ => false,
        }
    }

    fn validate<E: serde::de::Error>(value: &Value) -> Result<(), E> {
        use serde::de::Unexpected;
        let unexpected = match value {
            Value::Bool(_) | Value::Object(_) => return Ok(()),
            Value::Null => Unexpected::Unit,
            Value::Number(n) => {
                if let Some(u) = n.as_u64() {
                    Unexpected::Unsigned(u)
                } else if let Some(i) = n.as_i64() {
                    Unexpected::Signed(i)
                } else if let Some(f) = n.as_f64() {
                    Unexpected::Float(f)
                } else {
                    unreachable!()
                }
            }
            Value::String(s) => Unexpected::Str(s),
            Value::Array(_) => Unexpected::Seq,
        };

        Err(E::invalid_type(unexpected, &"object or boolean"))
    }

    #[allow(unsafe_code)]
    #[ref_cast_custom]
    fn ref_cast(value: &Value) -> &Self;

    #[allow(unsafe_code)]
    #[ref_cast_custom]
    fn ref_cast_mut(value: &mut Value) -> &mut Self;
}

impl From<Schema> for Value {
    fn from(v: Schema) -> Value {
        v.0
    }
}

impl std::convert::TryFrom<Value> for Schema {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> serde_json::Result<Schema> {
        Schema::validate(&value)?;
        Ok(Schema(value))
    }
}

impl<'a> std::convert::TryFrom<&'a Value> for &'a Schema {
    type Error = serde_json::Error;

    fn try_from(value: &Value) -> serde_json::Result<&Schema> {
        Schema::validate(value)?;
        Ok(Schema::ref_cast(value))
    }
}

impl<'a> std::convert::TryFrom<&'a mut Value> for &'a mut Schema {
    type Error = serde_json::Error;

    fn try_from(value: &mut Value) -> serde_json::Result<&mut Schema> {
        Schema::validate(value)?;
        Ok(Schema::ref_cast_mut(value))
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self(Value::Object(Map::new()))
    }
}

impl From<Map<String, Value>> for Schema {
    fn from(o: Map<String, Value>) -> Self {
        Schema(Value::Object(o))
    }
}

impl From<bool> for Schema {
    fn from(b: bool) -> Self {
        Schema(Value::Bool(b))
    }
}

impl crate::JsonSchema for Schema {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Schema".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "schemars::Schema".into()
    }

    fn json_schema(_: &mut crate::gen::SchemaGenerator) -> Schema {
        crate::json_schema!({
            "type": ["object", "boolean"]
        })
    }
}

mod ser {
    use serde::ser::{Serialize, SerializeMap, SerializeSeq};
    use serde_json::Value;

    // The order of properties in a JSON Schema object is insignificant, but we explicitly order
    // some of them here to make them easier for a human to read. All other properties are ordered
    // either lexicographically (by default) or by insertion order (if `preserve_order` is enabled)
    const ORDERED_KEYWORDS_START: [&str; 7] = [
        "$id",
        "$schema",
        "title",
        "description",
        "type",
        "format",
        "properties",
    ];
    const ORDERED_KEYWORDS_END: [&str; 2] = ["$defs", "definitions"];

    pub(super) struct OrderedKeywordWrapper<'a>(pub &'a Value);

    impl Serialize for OrderedKeywordWrapper<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match self.0 {
                Value::Array(array) => {
                    let mut seq = serializer.serialize_seq(Some(array.len()))?;
                    for value in array {
                        seq.serialize_element(&OrderedKeywordWrapper(value))?;
                    }
                    seq.end()
                }
                Value::Object(object) => {
                    let mut map = serializer.serialize_map(Some(object.len()))?;

                    for key in ORDERED_KEYWORDS_START {
                        if let Some(value) = object.get(key) {
                            map.serialize_entry(key, &OrderedKeywordWrapper(value))?;
                        }
                    }

                    for (key, value) in object {
                        if !ORDERED_KEYWORDS_START.contains(&key.as_str())
                            && !ORDERED_KEYWORDS_END.contains(&key.as_str())
                        {
                            map.serialize_entry(key, &OrderedKeywordWrapper(value))?;
                        }
                    }

                    for key in ORDERED_KEYWORDS_END {
                        if let Some(value) = object.get(key) {
                            map.serialize_entry(key, &OrderedKeywordWrapper(value))?;
                        }
                    }

                    map.end()
                }
                _ => self.0.serialize(serializer),
            }
        }
    }
}
