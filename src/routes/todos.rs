use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
/*use sqlx::{FromRow, MySqlPool};

#[derive(Serialize, Deserialize, FromRow)]
pub struct CreateTodo {
    title: String,
    description: Option<String>,
}
// TODO
#[derive(Serialize, Deserialize, FromRow)]
pub struct Todo {
    id: i32,
    title: String,
    description: Option<String>,
    status: String,
}
impl Todo {
    pub async fn get_all_todos(State(db): State<MySqlPool>) -> impl IntoResponse {
        let res: Result<Vec<Self>, sqlx::Error> = sqlx::query_as(
            "SELECT id, title, description, status
            from todos
          ",
        )
        .fetch_all(&db)
        .await;

        match res {
            Ok(todos) => (StatusCode::OK, Json(todos)).into_response(),
            Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, _e.to_string()).into_response(),
        }
    }

    pub async fn create_a_todo(
        State(db): State<MySqlPool>,
        Json(body): Json<CreateTodo>,
    ) -> impl IntoResponse {
        let res = sqlx::query(
            "INSERT INTO db1
        (title, description) values(?,?)
        ",
        )
        .bind(&body.title)
        .bind(&body.description)
        .execute(&db)
        .await;
        match res {
            Ok(todo) => (
                StatusCode::CREATED,
                Json(Self {
                    id: todo.last_insert_id() as i32,
                    description: body.description.clone(),
                    status: "New".to_string(),
                    title: body.title.clone(),
                }),
            )
                .into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}
*/
