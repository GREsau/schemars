use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;

// TODO use serde_json::Map (or some other wrapper) instead of BTreeMap to ensure preserve_order is possible

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum Schema {
    Bool(bool),
    Ref(SchemaRef),
    Object(SchemaObject),
}

impl From<SchemaObject> for Schema {
    fn from(o: SchemaObject) -> Self {
        Schema::Object(o)
    }
}

impl From<bool> for Schema {
    fn from(b: bool) -> Self {
        Schema::Bool(b)
    }
}

impl From<SchemaRef> for Schema {
    fn from(r: SchemaRef) -> Self {
        Schema::Ref(r)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub reference: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaObject {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub instance_type: Option<SingleOrVec<InstanceType>>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<SingleOrVec<Schema>>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub properties: Map<String, Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub definitions: Map<String, Schema>,
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum InstanceType {
    Null,
    Boolean,
    Object,
    Array,
    Number,
    String,
    Integer,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Single(Box<T>),
    Vec(Vec<T>),
}

impl<T> From<T> for SingleOrVec<T> {
    fn from(single: T) -> Self {
        SingleOrVec::Single(Box::new(single))
    }
}

impl<T> From<Vec<T>> for SingleOrVec<T> {
    fn from(vec: Vec<T>) -> Self {
        SingleOrVec::Vec(vec)
    }
}
