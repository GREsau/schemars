use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub reference: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
    pub items: Option<Box<Schema>>,
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
    pub extra_properties: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SingleOrVec<T> {
    Single(T),
    Vec(Vec<T>),
}

impl<T> From<T> for SingleOrVec<T> {
    fn from(single: T) -> Self {
        SingleOrVec::Single(single)
    }
}

impl<T> From<Vec<T>> for SingleOrVec<T> {
    fn from(mut vec: Vec<T>) -> Self {
        match vec.len() {
            1 => SingleOrVec::Single(vec.remove(0)),
            _ => SingleOrVec::Vec(vec),
        }
    }
}

impl<T> Into<Vec<T>> for SingleOrVec<T> {
    fn into(self) -> Vec<T> {
        match self {
            SingleOrVec::Single(s) => vec![s],
            SingleOrVec::Vec(v) => v,
        }
    }
}
