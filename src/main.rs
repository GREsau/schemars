mod make_schema;
mod schema;

use make_schema::MakeSchema;
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

fn main() -> Result<()> {
    let schema = <&str>::make_schema();
    let json = serde_json::to_string(&schema)?;
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
