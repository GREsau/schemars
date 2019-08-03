use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap as Map;

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RootSchema {
    #[serde(rename = "$schema", skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(flatten)]
    pub root: Schema,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "$id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub instance_type: Option<SingleOrVec<InstanceType>>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub instance_enum: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub definitions: Map<String, Schema>,
    #[serde(flatten)]
    pub extra_properties: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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

#[derive(Serialize, Deserialize, Debug)]
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

/*pub struct Schema {
    pub ref_path: Option<String>,
    pub description: Option<String>,
    pub schema_type: Option<String>,
    pub format: Option<String>,
    pub enum_values: Option<Vec<String>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<Schema>>,
    pub properties: Option<std::collections::BTreeMap<String, Schema>>,
}
*/
