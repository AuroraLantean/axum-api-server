use axum::{
    extract::path,
    //body,
    http::Method,
    middleware,
    routing::{get, patch, post, put},
    Extension,
    Router,
};
use dotenvy::dotenv;
use route_func::*;
use tower_http::cors::{Any, CorsLayer};

mod route_func;

//mod todos;//MySQL
//use todos::*;//MySQL
mod database;
use database::*;

pub async fn create_routes() -> Router {
    dotenv().ok();
    let db_postgres_uri = dotenvy::var("DB_POSTGRES_URL").unwrap();
    //let db_postgres_uri = dotenv!("DB_POSTGRES_URL");
    let db_conn = connect_db(db_postgres_uri.as_str())
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
    // Extension(config), Extension(db_conn) MUST be below any routes to make data available to them
    //place get_custom_middleware route to 1st route, and set_custom_middleware as 2nd route, so set_custom_middleware will only run before get_custom_middleware(the route above it)!!!
    Router::new()
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
        .route("/always_errors", get(always_errors))
        .route("/validate_struct_input", post(validate_struct_input))
        .route("/add_task", post(add_task))
        .route("/tasks/:id", get(get_task_by_id))
        .route("/tasks", get(get_tasks_all))
        .route("/tasks/:id", put(replace_task))
        .route("/tasks/:id", patch(update_partial_task))
        //.route("/todos/all", get(Todo::get_all_todos))
        //.route("/todo/create", post(Todo::create_a_todo))
        .route("/users", post(create_user))
        .layer(Extension(config))
        .layer(cors)
        .layer(Extension(db_conn))
    //.with_state(db)
}
