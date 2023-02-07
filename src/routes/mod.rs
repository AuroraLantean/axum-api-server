use axum::{
    http::Method,
    middleware,
    routing::{delete, get, patch, post, put},
    Extension, Router,
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
    let db_postgres_uri = dotenvy::var("DB_POSTGRES_URL").expect("postgres url not found");
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
    //logout must have auth to continue
    //move hello up to test jwt to avoid logout every single time
    //test jwt by waiting pass duration time
    //route_layer can ONLY BE ONE!!
    Router::new()
        .route("/users/logout", post(logout))
        .route("/add_task", post(add_task))
        .route("/hello", get(hello))
        .route_layer(middleware::from_fn(auth))
        //.route("/get_custom_middleware", get(get_custom_middleware))
        //.route_layer(middleware::from_fn(set_custom_middleware))
        .route("/", get(root))
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
        .route("/users", post(add_user))
        .route("/users/login", post(login))
        .route("/tasks/:id", get(get_task_by_id))
        .route("/tasks", get(get_tasks_all))
        .route("/tasks/:id", put(replace_task))
        .route("/tasks/:id", patch(update_partial_task))
        .route("/tasks/:id", delete(delete_task))
        //.route("/todos/all", get(Todo::get_all_todos))
        //.route("/todo/create", post(Todo::create_a_todo))
        .layer(Extension(config))
        .layer(cors)
        .layer(Extension(db_conn))
    //.with_state(db)
}
