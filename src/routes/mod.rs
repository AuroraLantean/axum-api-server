use axum::{
    extract::FromRef,
    http::Method,
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use route_func::*;
use sea_orm::DatabaseConnection;
use tower_http::cors::{Any, CorsLayer};

mod route_func;

// get_appstate: MUST have Clone macro!
// to auto extract fields: cargo add axum -F macro, add FromRef in macros below, then see get_appstate_mode()
#[derive(Clone, FromRef)]
pub struct AppState {
    pub mode: String,
    pub db_conn: DatabaseConnection,
}

pub async fn create_routes(mode: String, db_conn: DatabaseConnection) -> Router {
    let app_state = AppState { db_conn, mode };
    //to intercept incoming calls from untrusted brower origins
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any);
    // confirm active by seeing access-control-allow-origin from response headers

    // with_state(db_conn) MUST be below any routes to make data available to them
    //logout must have auth to continue
    //move hello up to test jwt to avoid logout every single time
    //test jwt by waiting pass duration time
    //route_layer can ONLY BE ONE!!
    Router::new()
        .route("/users/logout", post(logout))
        .route("/add_task", post(add_task))
        .route("/hello", get(hello))
        .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
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
        .route("/get_appstate", get(get_appstate))
        .route("/get_appstate_mode", get(get_appstate_mode))
        .route("/always_errors", get(always_errors))
        .route("/validate_struct_input", post(validate_struct_input))
        .route("/users", post(add_user))
        .route("/users/login", post(login))
        .route("/tasks/:id", get(get_task_by_id))
        .route("/tasks", get(get_tasks_all))
        .route("/tasks/:id", put(replace_task))
        .route("/tasks/:id", patch(update_partial_task))
        .route("/tasks/:id", delete(delete_task))
        .route("/eth_local_txn", post(eth_local_txn))
        .route("/eth_deploy_contract", post(eth_deploy_contract))
        .route("/eth_live_read", post(eth_live_read))
        .route("/eth_live_write", post(eth_live_write))
        .route("/eth_send_ether", post(eth_send_ether))
        .route("/chainlink_prices", get(chainlink_prices))
        .route("/run_thread", post(run_thread))
        .route("/make_post_request", post(make_post_request))
        .route("/make_get_request", get(make_get_request))
        .route("/download_file", post(download_file))
        .layer(cors)
        .with_state(app_state)
}
