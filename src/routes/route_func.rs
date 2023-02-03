use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, Path, Query, State},
    headers::UserAgent,
    http::{self, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
    response::{Html, IntoResponse},
    BoxError, Extension, Json, RequestExt, TypedHeader,
};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::entities::tasks;

/*use sqlx::MySqlPool;
// basic handler that responds with a static string
pub async fn root(State(_db): State<MySqlPool>) -> &'static str {
    "root"
}
pub async fn hello(State(_db): State<MySqlPool>) -> &'static str {
    "Hello, World!"
}*/
pub async fn root() -> &'static str {
    "root"
}
pub async fn hello() -> &'static str {
    "Hello, World!"
}
pub async fn send_html() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
//curl -X POST 127.0.0.1:3000/get_body_string
pub async fn get_body_string(body: String) -> String {
    body
}
// exactmatch must not take "Path" as argument bcos it is exact match already!
pub async fn exactmatch() -> impl IntoResponse {
    Json(User {
        username: "exactmatch will take priority".to_owned(),
        user_id: 092,
    })
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryParams {
    user_id: u64,
    code: String,
}
pub async fn query_params(Query(query): Query<QueryParams>) -> impl IntoResponse {
    Json(query)
}

// get_config
#[derive(Clone)]
pub struct Config {
    pub mode: String,
}
pub async fn get_config(Extension(config): Extension<Config>) -> String {
    config.mode
}
pub async fn always_errors() -> Result<(), StatusCode> {
    Err(StatusCode::IM_A_TEAPOT)
    //Ok(())
}

// needs "headers" feature from axum
pub async fn query_headers(TypedHeader(user_agent): TypedHeader<UserAgent>) -> String {
    user_agent.to_string()
}
// needs "headers" feature from axum
pub async fn query_custom_headers(headers: HeaderMap) -> String {
    let auth_headervalue = headers.get("Authorization").unwrap(); //can be used for "User-Agent"
    auth_headervalue.to_str().unwrap().to_owned()
}

#[derive(Clone)] //ADD Clone to avoid trait bound error
pub struct SecurityLevel(pub String);
//add "pub" inside OR "cannot initialize a tuple struct which contains private field"

pub async fn get_custom_middleware(Extension(security_level): Extension<SecurityLevel>) -> String {
    security_level.0
}
//https://docs.rs/axum/latest/axum/middleware/index.html#writing-middleware
pub async fn set_custom_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let headers = request.headers();
    //let security_level = headers.get("security-level").unwrap(); // will crash the server if error happens!
    let security_level = headers
        .get("security-level")
        .ok_or_else(|| StatusCode::BAD_REQUEST)?;
    let security_level = security_level
        .to_str()
        .map_err(|_error| StatusCode::BAD_REQUEST)?
        .to_owned();
    let extensions = request.extensions_mut();
    //make separate line OR cannot borrow as mutable...also borrowed as immutable
    extensions.insert(SecurityLevel(security_level));
    Ok(next.run(request).await)
}

//2xx: all ok. 201 success for created item
//3xx: redirect ok
//4xx: errors from the client, 401 and 403 are auth based
//5xx: errors from the server

//201 means success at created item

#[derive(Deserialize, Debug, Validate)]
pub struct AddUser {
    pub username: String,
    #[validate(length(min = 8, message = "must have at least 8 characters"))]
    pub password: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    pub nickname: Option<String>,
} //Option field in input struct
  //https://github.com/Keats/validator

//Custom Extractor to validate struct input
//https://docs.rs/axum/latest/axum/extract/trait.FromRequest.html
#[async_trait]
impl<S, B> FromRequest<S, B> for AddUser
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(request: Request<B>, _state: &S) -> Result<Self, Self::Rejection> {
        let Json(user) = request
            .extract::<Json<AddUser>, _>()
            .await
            .map_err(|error| (StatusCode::BAD_REQUEST, format!("{}", error)))?;

        if let Err(errors) = user.validate() {
            return Err((StatusCode::BAD_REQUEST, format!("{}", errors)));
        }
        Ok(user)
    }
}
#[derive(Serialize)]
pub struct Output<'a> {
    error_code: u32,
    message: &'a str,
}
pub async fn validate_struct_input(body: AddUser) -> impl IntoResponse {
    dbg!(body);
    Json(Output {
        error_code: 0,
        message: "ok",
    })
}
/*pub async fn validate_struct_input(Json(body): Json<AddUser>) -> impl IntoResponse {
    dbg!(&body);
    (StatusCode::CREATED, "new user added".to_owned()).into_response()
}
-> Response {
    (StatusCode::CREATED, "new user added".to_owned()).into_response()}
*/

// Serialize for output body
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    user_id: u64,
    username: String,
}
//curl localhost:3000/user/9
pub async fn get_user_by_id(Path(user_id): Path<u64>) -> impl IntoResponse {
    Json(User {
        username: "get_user_by_id".to_owned(),
        user_id,
    })
}

// Deserialize for input body, Debug for terminal print
#[derive(Deserialize, Debug)]
pub struct CreateTask {
    pub title: String,
    pub priority: Option<String>,
    pub description: Option<String>,
}
pub async fn create_task(
    Extension(db_conn): Extension<DatabaseConnection>,
    Json(payload): Json<CreateTask>,
) -> impl IntoResponse {
    let new_task = tasks::ActiveModel {
        title: Set(payload.title),
        priority: Set(payload.priority),
        description: Set(payload.description),
        ..Default::default()
    }; // get all fields by clicking at the error warning at the lower left of VSCode, then right clicking on the error message there
    let active_model = new_task.save(&db_conn).await.unwrap();
    dbg!(active_model);
    Json(Output {
        error_code: 0,
        message: concat!("ok", "foo"),
    })
}
//------------------==
#[derive(Deserialize, Debug)]
pub struct CreateUser {
    username: String,
}
pub async fn create_user(
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
