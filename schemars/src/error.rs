use std::fmt;
use std::error::Error;
use crate::schema::Schema;

pub type Result<T = Schema> = std::result::Result<T, JsonSchemaError>;

#[derive(Debug, Clone)]
pub struct JsonSchemaError {
    msg: &'static str,
    schema: Schema
}

impl JsonSchemaError {
    pub fn new(msg: &'static str, schema: Schema) -> JsonSchemaError {
        JsonSchemaError { msg, schema }
    }
}

impl fmt::Display for JsonSchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} Schema: {:?}", self.msg, self.schema)
    }
}

impl Error for JsonSchemaError {
}
