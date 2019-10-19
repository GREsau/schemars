use crate::schema::Schema;
use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, JsonSchemaError>;

#[derive(Debug, Clone)]
pub struct JsonSchemaError {
    msg: &'static str,
    schema: Schema,
}

impl JsonSchemaError {
    pub fn new(msg: &'static str, schema: Schema) -> JsonSchemaError {
        JsonSchemaError { msg, schema }
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

impl fmt::Display for JsonSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Schema: {:?}", self.msg, self.schema)
    }
}

impl Error for JsonSchemaError {}
