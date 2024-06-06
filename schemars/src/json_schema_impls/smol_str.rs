use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

#[cfg(feature = "smol_str01")]
forward_impl!(smol_str01::SmolStr => String);
#[cfg(feature = "smol_str02")]
forward_impl!(smol_str02::SmolStr => String);
