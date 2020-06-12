use crate::schema::{RootSchema, Schema, SchemaObject, SingleOrVec};
use std::fmt::Debug;

/// TODO document
pub trait Visitor {
    /// TODO document
    fn visit_root_schema(&mut self, root: &mut RootSchema) {
        visit_root_schema(self, root)
    }

    /// TODO document
    fn visit_schema(&mut self, schema: &mut Schema) {
        visit_schema(self, schema)
    }

    /// TODO document
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        visit_schema_object(self, schema)
    }
}

/// TODO document
pub fn visit_root_schema<V: Visitor + ?Sized>(v: &mut V, root: &mut RootSchema) {
    v.visit_schema_object(&mut root.schema);
    visit_map_values(v, &mut root.definitions);
}

/// TODO document
pub fn visit_schema<V: Visitor + ?Sized>(v: &mut V, schema: &mut Schema) {
    if let Schema::Object(schema) = schema {
        v.visit_schema_object(schema)
    }
}

/// TODO document
pub fn visit_schema_object<V: Visitor + ?Sized>(v: &mut V, schema: &mut SchemaObject) {
    if let Some(sub) = &mut schema.subschemas {
        visit_vec(v, &mut sub.all_of);
        visit_vec(v, &mut sub.any_of);
        visit_vec(v, &mut sub.one_of);
        visit_box(v, &mut sub.not);
        visit_box(v, &mut sub.if_schema);
        visit_box(v, &mut sub.then_schema);
        visit_box(v, &mut sub.else_schema);
    }

    if let Some(arr) = &mut schema.array {
        visit_single_or_vec(v, &mut arr.items);
        visit_box(v, &mut arr.additional_items);
        visit_box(v, &mut arr.contains);
    }

    if let Some(obj) = &mut schema.object {
        visit_map_values(v, &mut obj.properties);
        visit_map_values(v, &mut obj.pattern_properties);
        visit_box(v, &mut obj.additional_properties);
        visit_box(v, &mut obj.property_names);
    }
}

fn visit_box<V: Visitor + ?Sized>(v: &mut V, target: &mut Option<Box<Schema>>) {
    if let Some(s) = target {
        v.visit_schema(s)
    }
}

fn visit_vec<V: Visitor + ?Sized>(v: &mut V, target: &mut Option<Vec<Schema>>) {
    if let Some(vec) = target {
        for s in vec {
            v.visit_schema(s)
        }
    }
}

fn visit_map_values<V: Visitor + ?Sized>(v: &mut V, target: &mut crate::Map<String, Schema>) {
    for s in target.values_mut() {
        v.visit_schema(s)
    }
}

fn visit_single_or_vec<V: Visitor + ?Sized>(v: &mut V, target: &mut Option<SingleOrVec<Schema>>) {
    match target {
        None => {}
        Some(SingleOrVec::Single(s)) => v.visit_schema(s),
        Some(SingleOrVec::Vec(vec)) => {
            for s in vec {
                v.visit_schema(s)
            }
        }
    }
}

/// TODO document
#[derive(Debug, Clone)]
pub struct ReplaceBoolSchemas {
    /// TODO document
    pub skip_additional_properties: bool,
}

impl Visitor for ReplaceBoolSchemas {
    fn visit_schema(&mut self, schema: &mut Schema) {
        visit_schema(self, schema);

        if let Schema::Bool(b) = *schema {
            *schema = Schema::Bool(b).into_object().into()
        }
    }

    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        if self.skip_additional_properties {
            if let Some(obj) = &mut schema.object {
                if let Some(ap) = &obj.additional_properties {
                    if let Schema::Bool(_) = ap.as_ref() {
                        let additional_properties = obj.additional_properties.take();
                        visit_schema_object(self, schema);
                        schema.object().additional_properties = additional_properties;

                        return;
                    }
                }
            }
        }

        visit_schema_object(self, schema);
    }
}

/// TODO document
#[derive(Debug, Clone)]
pub struct RemoveRefSiblings;

impl Visitor for RemoveRefSiblings {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        visit_schema_object(self, schema);

        if let Some(reference) = schema.reference.take() {
            if schema == &SchemaObject::default() {
                schema.reference = Some(reference);
            } else {
                let ref_schema = Schema::new_ref(reference);
                let all_of = &mut schema.subschemas().all_of;
                match all_of {
                    Some(vec) => vec.push(ref_schema),
                    None => *all_of = Some(vec![ref_schema]),
                }
            }
        }
    }
}

/// TODO document
#[derive(Debug, Clone)]
pub struct SetSingleExample {
    /// TODO document
    pub retain_examples: bool,
}

impl Visitor for SetSingleExample {
    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        visit_schema_object(self, schema);

        let first_example = schema.metadata.as_mut().and_then(|m| {
            if self.retain_examples {
                m.examples.first().cloned()
            } else {
                m.examples.drain(..).next()
            }
        });

        if let Some(example) = first_example {
            schema.extensions.insert("example".to_owned(), example);
        }
    }
}
