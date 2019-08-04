pub mod generator;
pub mod make_schema;
pub mod schema;

use make_schema::MakeSchema;
use schema::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum TodoStatus {
    Backlog,
    InProgress,
    Done,
    Archived,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Todo {
    id: u64,
    title: String,
    description: Option<String>,
    status: TodoStatus,
    assigned_to: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct User {
    id: u64,
    username: String,
}

impl MakeSchema for User {
    fn generates_ref_schema() -> bool {
        true
    }

    fn make_schema(gen: &mut generator::SchemaGenerator) -> Schema {
        let mut o = SchemaObject {
            ..Default::default()
        };
        o.properties
            .insert("id".to_owned(), gen.subschema_for::<u64>());
        o.properties
            .insert("username".to_owned(), gen.subschema_for::<String>());
        o.into()
    }
}

fn main() -> Result<()> {
    let gen = generator::SchemaGenerator::new();
    let schema = gen.into_root_schema_for::<User>();
    let json = serde_json::to_string_pretty(&schema)?;
    println!("{}", json);

    /*let todo = Todo {
        id: 42,
        title: "Learn Rust".to_owned(),
        description: Option::None,
        status: TodoStatus::InProgress,
        assigned_to: vec![User {
            id: 1248,
            username: "testuser".to_owned(),
        }],
    };

    let t = serde_json::to_string(&todo)?;
    println!("{}", t);*/

    Ok(())
}
