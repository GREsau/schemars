use crate as schemars;
use crate::{JsonSchema, JsonSchemaError, Map, Result, Set};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Schema {
    Bool(bool),
    Ref(Ref),
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

impl From<Ref> for Schema {
    fn from(r: Ref) -> Self {
        Schema::Ref(r)
    }
}

impl Schema {
    pub fn flatten(self, other: Self) -> Result {
        fn extend<A, E: Extend<A>>(mut a: E, b: impl IntoIterator<Item = A>) -> E {
            a.extend(b);
            a
        }

        let s1 = self.ensure_flattenable()?;
        let s2 = other.ensure_flattenable()?;
        Ok(Schema::Object(SchemaObject {
            schema: s1.schema.or(s2.schema),
            id: s1.id.or(s2.id),
            title: s1.title.or(s2.title),
            description: s1.description.or(s2.description),
            definitions: extend(s1.definitions, s2.definitions),
            extensions: extend(s1.extensions, s2.extensions),
            // TODO do the following make sense?
            instance_type: s1.instance_type.or(s2.instance_type),
            format: s1.format.or(s2.format),
            enum_values: s1.enum_values.or(s2.enum_values),
            all_of: s1.all_of.or(s2.all_of),
            any_of: s1.any_of.or(s2.any_of),
            one_of: s1.one_of.or(s2.one_of),
            not: s1.not.or(s2.not),
            if_schema: s1.if_schema.or(s2.if_schema),
            then_schema: s1.then_schema.or(s2.then_schema),
            else_schema: s1.else_schema.or(s2.else_schema),
            number: NumberValidation {
                multiple_of: s1.number.multiple_of.or(s2.number.multiple_of),
                maximum: s1.number.maximum.or(s2.number.maximum),
                exclusive_maximum: s1.number.exclusive_maximum.or(s2.number.exclusive_maximum),
                minimum: s1.number.minimum.or(s2.number.minimum),
                exclusive_minimum: s1.number.exclusive_minimum.or(s2.number.exclusive_minimum),
            },
            string: StringValidation {
                max_length: s1.string.max_length.or(s2.string.max_length),
                min_length: s1.string.min_length.or(s2.string.min_length),
                pattern: s1.string.pattern.or(s2.string.pattern),
            },
            array: ArrayValidation {
                items: s1.array.items.or(s2.array.items),
                additional_items: s1.array.additional_items.or(s2.array.additional_items),
                max_items: s1.array.max_items.or(s2.array.max_items),
                min_items: s1.array.min_items.or(s2.array.min_items),
                unique_items: s1.array.unique_items.or(s2.array.unique_items),
                contains: s1.array.contains.or(s2.array.contains),
            },
            object: ObjectValidation {
                max_properties: s1.object.max_properties.or(s2.object.max_properties),
                min_properties: s1.object.min_properties.or(s2.object.min_properties),
                required: extend(s1.object.required, s2.object.required),
                properties: extend(s1.object.properties, s2.object.properties),
                pattern_properties: extend(s1.object.pattern_properties, s2.object.pattern_properties),
                additional_properties: s1.object.additional_properties.or(s2.object.additional_properties),
                property_names: s1.object.property_names.or(s2.object.property_names),
            },
        }))
    }

    fn ensure_flattenable(self) -> Result<SchemaObject> {
        let s = match self {
            Schema::Object(s) => s,
            s => {
                return Err(JsonSchemaError::new(
                    "Only schemas with type `object` can be flattened.",
                    s,
                ))
            }
        };
        match s.instance_type {
            Some(SingleOrVec::Single(ref t)) if **t != InstanceType::Object => {
                Err(JsonSchemaError::new(
                    "Only schemas with type `object` can be flattened.",
                    s.into(),
                ))
            }
            Some(SingleOrVec::Vec(ref t)) if !t.contains(&InstanceType::Object) => {
                Err(JsonSchemaError::new(
                    "Only schemas with type `object` can be flattened.",
                    s.into(),
                ))
            }
            _ => Ok(s),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct Ref {
    #[serde(rename = "$ref")]
    pub reference: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_of: Option<Vec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not: Option<Box<Schema>>,
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub if_schema: Option<Box<Schema>>,
    #[serde(rename = "then", skip_serializing_if = "Option::is_none")]
    pub then_schema: Option<Box<Schema>>,
    #[serde(rename = "else", skip_serializing_if = "Option::is_none")]
    pub else_schema: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub definitions: Map<String, Schema>,
    #[serde(flatten)]
    pub number: NumberValidation,
    #[serde(flatten)]
    pub string: StringValidation,
    #[serde(flatten)]
    pub array: ArrayValidation,
    #[serde(flatten)]
    pub object: ObjectValidation,
    #[serde(flatten)]
    pub extensions: Map<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct NumberValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multiple_of: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_maximum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclusive_minimum: Option<f64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct StringValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ArrayValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<SingleOrVec<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_items: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique_items: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Box<Schema>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ObjectValidation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_properties: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_properties: Option<u32>,
    #[serde(skip_serializing_if = "Set::is_empty")]
    pub required: Set<String>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub properties: Map<String, Schema>,
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub pattern_properties: Map<String, Schema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<Box<Schema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_names: Option<Box<Schema>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
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
