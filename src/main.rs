use axum::{
    body,
    extract::{Path, Query, State},
    headers::UserAgent,
    http::{request, HeaderMap, Method, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router, TypedHeader,
};
use serde::{Deserialize, Serialize};
use sqlx::{MySql, MySqlPool};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

mod database;
use database::*;
mod todos;
use todos::*;
//use root_package_name::run;
//mod route_abc;//import from src/routes/route_abc.rs
//use route_abc::*;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    let db = database_connection()
        .await
        .expect("failed to connect to database");

    //to intercept incoming calls from untrusted brower origins
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);
    // confirm active by seeing access-control-allow-origin from response headers

    let config = Config {
        mode: "normal".to_owned(),
    };
    // Extension(config) MUST be below any routes to make config available to them
    //set_custom_middleware will run only before what is above it! So
    let app = Router::new()
        .route("/get_custom_middleware", get(get_custom_middleware))
        .route_layer(middleware::from_fn(set_custom_middleware))
        .route("/", get(root))
        .route("/hello", get(hello))
        .route("/html", get(send_html))
        .route("/get_body_string", post(get_body_string))
        .route("/user/:id", get(get_user_by_id))
        .route("/user/092", get(exactmatch))
        .route("/query_params", get(query_params))
        .route("/query_headers", get(query_headers))
        .route("/query_custom_headers", get(query_custom_headers))
        .route("/get_config", get(get_config))
        .route("/user/struct_input_output", post(struct_input_output))
        .route("/todos/all", get(Todo::get_all_todos))
        .route("/todo/create", post(Todo::create_a_todo))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user))
        .layer(Extension(config))
        .layer(cors)
        .with_state(db);

    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // 0.0.0.0 is compatible for docker containers and VM
    tracing::debug!("listening on {}", addr);
    println!("Server running on localhost:3000");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root(State(_db): State<MySqlPool>) -> &'static str {
    "root"
}
async fn hello(State(_db): State<MySqlPool>) -> &'static str {
    "Hello, World!"
}
async fn send_html() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
//curl -X POST 127.0.0.1:3000/get_body_string
async fn get_body_string(body: String) -> String {
    body
}
// exactmatch must not take "Path" as argument bcos it is exact match already!
async fn exactmatch() -> impl IntoResponse {
    Json(User {
        username: "exactmatch will take priority".to_owned(),
        user_id: 092,
    })
}
#[derive(Serialize, Deserialize, Debug)]
struct QueryParams {
    user_id: u64,
    code: String,
}
async fn query_params(Query(query): Query<QueryParams>) -> impl IntoResponse {
    Json(query)
}

// needs "headers" feature from axum
async fn query_headers(TypedHeader(user_agent): TypedHeader<UserAgent>) -> String {
    user_agent.to_string()
}
// needs "headers" feature from axum
async fn query_custom_headers(headers: HeaderMap) -> String {
    let auth = headers.get("Authorization").unwrap(); //can be used for "User-Agent"
    auth.to_str().unwrap().to_owned()
}

// get_config
#[derive(Clone)]
pub struct Config {
    pub mode: String,
}
async fn get_config(Extension(config): Extension<Config>) -> String {
    config.mode
}

#[derive(Clone)]
pub struct SecurityLevel(pub String);
async fn get_custom_middleware(Extension(security_level): Extension<SecurityLevel>) -> String {
    security_level.0
}

async fn set_custom_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    let security_level = headers
        .get("security-level")
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let security_level = security_level
        .to_str()
        .map_err(|_error| StatusCode::BAD_REQUEST)?
        .to_owned();
    let extensions = request.extensions_mut();
    extensions.insert(SecurityLevel(security_level));
    Ok(next.run(request).await)
}

//CORS https://docs.rs/tower-http/latest/tower_http/cors/index.html

//curl -X POST 127.0.0.1:3000
async fn struct_input_output(Json(body): Json<User>) -> impl IntoResponse {
    dbg!(&body);
    Json(User {
        user_id: body.user_id,
        username: body.username,
    })
}
// Serialize for output body
#[derive(Serialize, Deserialize, Debug)]
struct User {
    user_id: u64,
    username: String,
}
//curl localhost:3000/user/9
async fn get_user_by_id(Path(user_id): Path<u64>) -> impl IntoResponse {
    Json(User {
        username: "get_user_by_id".to_owned(),
        user_id,
    })
}
// Deserialize for input body, Debug for terminal print
#[derive(Deserialize, Debug)]
struct CreateUser {
    username: String,
}
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    // insert your application logic here
    let user = User {
        user_id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

/*
//! API will be:
//!
//! - `GET /todos`: return a JSON list of Todos.
//! - `POST /todos`: create a new Todo.
//! - `PUT /todos/:id`: update a specific Todo.
//! - `DELETE /todos/:id`: delete a specific Todo.
//!
//! Run with
//!
//! ```not_rust
//! cd examples && cargo run -p example-todos
//! ```

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_todos=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Db::default();

    // Compose the routes
    let app = Router::new()
        .route("/todos", get(todos_index).post(todos_create))
        .route("/todos/:id", patch(todos_update).delete(todos_delete))
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            format!("Unhandled internal error: {}", error),
                        ))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        )
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// The query parameters for todos index
#[derive(Debug, Deserialize, Default)]
pub struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

async fn todos_index(
    pagination: Option<Query<Pagination>>,
    State(db): State<Db>,
) -> impl IntoResponse {
    let todos = db.read().unwrap();

    let Query(pagination) = pagination.unwrap_or_default();

    let todos = todos
        .values()
        .skip(pagination.offset.unwrap_or(0))
        .take(pagination.limit.unwrap_or(usize::MAX))
        .cloned()
        .collect::<Vec<_>>();

    Json(todos)
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    text: String,
}

async fn todos_create(State(db): State<Db>, Json(input): Json<CreateTodo>) -> impl IntoResponse {
    let todo = Todo {
        id: Uuid::new_v4(),
        text: input.text,
        completed: false,
    };

    db.write().unwrap().insert(todo.id, todo.clone());

    (StatusCode::CREATED, Json(todo))
}

#[derive(Debug, Deserialize)]
struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

async fn todos_update(
    Path(id): Path<Uuid>,
    State(db): State<Db>,
    Json(input): Json<UpdateTodo>,
) -> Result<impl IntoResponse, StatusCode> {
    let mut todo = db
        .read()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(text) = input.text {
        todo.text = text;
    }

    if let Some(completed) = input.completed {
        todo.completed = completed;
    }

    db.write().unwrap().insert(todo.id, todo.clone());

    Ok(Json(todo))
}

async fn todos_delete(Path(id): Path<Uuid>, State(db): State<Db>) -> impl IntoResponse {
    if db.write().unwrap().remove(&id).is_some() {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

type Db = Arc<RwLock<HashMap<Uuid, Todo>>>;

#[derive(Debug, Serialize, Clone)]
struct Todo {
    id: Uuid,
    text: String,
    completed: bool,
}

  */
