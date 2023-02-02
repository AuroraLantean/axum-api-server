use axum::{
    //body,
    http::Method,
    middleware,
    routing::{get, post},
    Extension,
    Router,
};
use tower_http::cors::{Any, CorsLayer};
mod route_func;
use route_func::*;
mod todos;
use todos::*;
mod database;
use database::*;

pub async fn create_routes() -> Router {
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
        .route("/add_user_successlly", post(add_user_successlly))
        .route("/validate_struct_input", post(validate_struct_input))
        .route("/user/struct_input_output", post(struct_input_output))
        .route("/todos/all", get(Todo::get_all_todos))
        .route("/todo/create", post(Todo::create_a_todo))
        .route("/users", post(create_user))
        .layer(Extension(config))
        .layer(cors)
        .with_state(db)
}
