use crate::schema::*;
use crate::{JsonSchemaError, Map, Result, Set};

impl Schema {
    pub fn flatten(self, other: Self) -> Result {
        if is_null_type(&self) {
            return Ok(other);
        } else if is_null_type(&other) {
            return Ok(self);
        }
        let s1 = ensure_object_type(self)?;
        let s2 = ensure_object_type(other)?;
        Ok(Schema::Object(s1.merge(s2)))
    }
}

trait Merge: Sized {
    fn merge(self, other: Self) -> Self;
}

macro_rules! impl_merge {
    ($ty:ident { merge: $($merge_field:ident)*, or: $($or_field:ident)*, }) => {
        impl Merge for $ty {
            fn merge(self, other: Self) -> Self {
                $ty {
                    $($merge_field: self.$merge_field.merge(other.$merge_field),)*
                    $($or_field: self.$or_field.or(other.$or_field),)*
                }
            }
        }
    };
    ($ty:ident { or: $($or_field:ident)*, }) => {
        impl_merge!( $ty { merge: , or: $($or_field)*, });
    };
}

impl_merge!(SchemaObject {
    merge: definitions extensions instance_type enum_values
        metadata subschemas number string array object,
    or: format const_value reference,
});

impl_merge!(Metadata {
    or: schema id title description,
});

impl_merge!(SubschemaValidation {
    or: all_of any_of one_of not if_schema then_schema else_schema,
});

impl_merge!(NumberValidation {
    or: multiple_of maximum exclusive_maximum minimum exclusive_minimum,
});

impl_merge!(StringValidation {
    or: max_length min_length pattern,
});

impl_merge!(ArrayValidation {
    or: items additional_items max_items min_items unique_items contains,
});

impl_merge!(ObjectValidation {
    merge: required properties pattern_properties,
    or: max_properties min_properties additional_properties property_names,
});

impl<T: Merge> Merge for Option<T> {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Some(x), Some(y)) => Some(x.merge(y)),
            (None, y) => y,
            (x, None) => x,
        }
    }
}

impl<T: Merge> Merge for Box<T> {
    fn merge(mut self, other: Self) -> Self {
        *self = (*self).merge(*other);
        self
    }
}

impl<T> Merge for Vec<T> {
    fn merge(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl<K: Ord, V> Merge for Map<K, V> {
    fn merge(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl<T: Ord> Merge for Set<T> {
    fn merge(mut self, other: Self) -> Self {
        self.extend(other);
        self
    }
}

impl Merge for SingleOrVec<InstanceType> {
    fn merge(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        let mut vec = match (self, other) {
            (SingleOrVec::Vec(v1), SingleOrVec::Vec(v2)) => v1.merge(v2),
            (SingleOrVec::Vec(mut v), SingleOrVec::Single(s))
            | (SingleOrVec::Single(s), SingleOrVec::Vec(mut v)) => {
                v.push(*s);
                v
            }
            (SingleOrVec::Single(s1), SingleOrVec::Single(s2)) => vec![*s1, *s2],
        };
        vec.sort();
        vec.dedup();
        SingleOrVec::Vec(vec)
    }
}

fn is_null_type(schema: &Schema) -> bool {
    let s = match schema {
        Schema::Object(s) => s,
        _ => return false,
    };
    match &s.instance_type {
        Some(SingleOrVec::Single(t)) if **t == InstanceType::Null => true,
        _ => false,
    }
}

fn ensure_object_type(schema: Schema) -> Result<SchemaObject> {
    let s = match schema {
        Schema::Object(s) => s,
        s => {
            return Err(JsonSchemaError::new(
                "Only schemas with type `object` or `null` can be flattened.",
                s,
            ))
        }
    };
    match s.instance_type {
        Some(SingleOrVec::Single(ref t)) if **t != InstanceType::Object => {
            Err(JsonSchemaError::new(
                "Only schemas with type `object` or `null` can be flattened.",
                s.into(),
            ))
        }
        Some(SingleOrVec::Vec(ref t)) if !t.contains(&InstanceType::Object) => {
            Err(JsonSchemaError::new(
                "Only schemas with type `object` or `null` can be flattened.",
                s.into(),
            ))
        }
        _ => Ok(s),
    }
}
