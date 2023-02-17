use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, Path, Query, State},
    headers::{authorization::Bearer, Authorization, UserAgent},
    http::{HeaderMap, Request, StatusCode},
    middleware::Next,
    response::Response,
    response::{Html, IntoResponse},
    BoxError, Extension, Json, RequestExt, TypedHeader,
};
use chrono::{DateTime, FixedOffset};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection,
    EntityTrait, IntoActiveModel, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::AppState;
use crate::blockchain::{contract_deploy::*, simple_txn_live::*, simple_txn_local::*};
use crate::{
    entities::{
        tasks::{self, Entity as Tasks},
        users::{self, Entity as Users, Model as UserModel},
    },
    utils::{hash_password, make_jwt, verify_jwt, verify_password, AppError},
};
use std::{collections::HashMap, time::Duration};
use std::{sync::mpsc, thread};
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

// State(..) will check if ".with_state(..) is in the routes"
pub async fn get_appstate(State(app_state): State<AppState>) -> String {
    app_state.mode
}
pub async fn get_appstate_mode(State(mode): State<String>) -> String {
    mode
}
pub async fn always_errors() -> Result<(), StatusCode> {
    Err(StatusCode::IM_A_TEAPOT)
    //Ok(())
}

// needs "headers" feature from axum
pub async fn query_headers(TypedHeader(user_agent): TypedHeader<UserAgent>) -> String {
    user_agent.to_string()
}

#[derive(Clone)] //ADD Clone to avoid trait bound error
pub struct SecurityLevel(pub String);
//add "pub" inside OR "cannot initialize a tuple struct which contains private field"

pub async fn _get_custom_middleware(Extension(security_level): Extension<SecurityLevel>) -> String {
    security_level.0
}
//https://docs.rs/axum/latest/axum/middleware/index.html#writing-middleware
pub async fn _set_custom_middleware<B>(
    mut request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    println!("----------== set_custom_middleware");
    let headers = request.headers();
    //let security_level = headers.get("security-level").unwrap(); // will crash the server if error happens!
    let security_level = headers
        .get("security-level")
        .ok_or(StatusCode::BAD_REQUEST)?;
    let security_level = security_level
        .to_str()
        .map_err(|_error| StatusCode::BAD_REQUEST)?
        .to_owned();
    let extensions = request.extensions_mut();
    //make separate line OR cannot borrow as mutable...also borrowed as immutable
    extensions.insert(SecurityLevel(security_level));
    Ok(next.run(request).await)
}
// needs "headers" feature from axum; custom extractor or mirror custom header
pub async fn query_custom_headers(headers: HeaderMap) -> String {
    let auth_headervalue = headers.get("Authorization").expect("err1"); //can be used for "User-Agent"
    auth_headervalue.to_str().expect("err2").to_owned()
}
//put your extractor after State(db_conn)
pub async fn auth<T>(
    State(db_conn): State<DatabaseConnection>,
    TypedHeader(token): TypedHeader<Authorization<Bearer>>,
    mut request: Request<T>,
    next: Next<T>,
) -> Result<Response, AppError> {
    println!("auth");
    let token = token.token().to_owned();
    // let token = request
    //     .headers()
    //     .typed_get::<Authorization<Bearer>>()
    //     .ok_or_else(|| {
    //         println!("auth err 101");
    //         AppError::new(StatusCode::BAD_REQUEST, "error 101")
    //     })?
    //     .token()
    //     .to_owned();
    dbg!(&token);

    // let db_conn = request
    //     .extensions()
    //     .get::<DatabaseConnection>()
    //     .ok_or_else(|| {
    //         println!("auth err 102");
    //         AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "error 102")
    //     })?;
    println!("db connected");

    let user = Users::find()
        .filter(users::Column::Token.eq(Some(token.clone())))
        .one(&db_conn)
        .await
        .map_err(|err| {
            println!("error 103: {err}");
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "error 103")
        })?;
    println!("user option is found");
    let Some(user) = user else {
        return Err(AppError::new(StatusCode::UNAUTHORIZED, "unauthorized. login or sign up. error 104"))
    };
    println!("user is valid");
    verify_jwt(&token)?; //move jwt verification here to confuse hackers so they don't know what is wrong out of jwt, db, or user
    println!("token is valid");

    request.extensions_mut().insert(user);
    Ok(next.run(request).await)
}
//2xx: all ok. 201 success for created item
//3xx: redirect ok
//4xx: errors from the client, 401 and 403 are auth based
//5xx: errors from the server

//201 means success at created item

//https://github.com/Keats/validator
#[derive(Deserialize, Debug, Validate)]
pub struct AddUser {
    pub username: String,
    #[validate(length(min = 8, message = "must have at least 8 characters"))]
    pub password: String,
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
    //pub legalname: Option<String>,
} //Option field in input struct
  //Custom Extractor to validate struct input
  //https://github.com/Keats/validator
#[derive(Serialize, Debug)]
pub struct ResponseAddUser {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub token: Option<String>,
    //pub deleted_at: Option<DateTime<FixedOffset>>,
}

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
        println!("----------== from_request");
        let Json(user) = request
            .extract::<Json<AddUser>, _>()
            .await
            .map_err(|error| (StatusCode::BAD_REQUEST, format!("{error}")))?;

        if let Err(errors) = user.validate() {
            return Err((StatusCode::BAD_REQUEST, format!("{errors}")));
        }
        Ok(user)
    }
}

pub async fn validate_struct_input(json: AddUser) -> impl IntoResponse {
    dbg!(json);
    (StatusCode::CREATED, "new user added".to_owned()).into_response()
}

pub async fn add_user(
    State(db_conn): State<DatabaseConnection>,
    //json: AddUser, //Must use this json format for validation!
    Json(json): Json<AddUser>,
) -> Result<Json<ResponseAddUser>, StatusCode> {
    dbg!(&json);
    let jwt_token = make_jwt()?;
    let new_user = users::ActiveModel {
        username: Set(json.username),
        password: Set(hash_password(json.password)?),
        token: Set(Some(jwt_token)),
        ..Default::default()
    }
    .save(&db_conn)
    .await
    .map_err(|err| {
        println!("saving new user failed:");
        dbg!(err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    dbg!(&new_user);
    Ok(Json(ResponseAddUser {
        user_id: new_user.id.unwrap(),
        username: new_user.username.unwrap(),
        email: json.email.to_owned(),
        token: new_user.token.unwrap(),
    }))
}
// State(..) will check if ".with_state(..) is in the routes"
#[derive(Deserialize, Debug, Validate)]
pub struct Login {
    pub username: String,
    #[validate(length(min = 8, message = "must have at least 8 characters"))]
    pub password: String,
} //Option field in input struct
pub async fn login(
    State(db_conn): State<DatabaseConnection>,
    Json(json): Json<Login>,
) -> Result<Json<ResponseAddUser>, StatusCode> {
    dbg!(&json);
    let db_user = Users::find()
        .filter(users::Column::Username.eq(json.username))
        .one(&db_conn)
        .await
        .map_err(|err| {
            println!("finding user failed:");
            dbg!(err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    dbg!(&db_user);

    if let Some(db_user) = db_user {
        if !verify_password(json.password, &db_user.password)? {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let jwt_token = make_jwt()?;
        println!("new jwt_token:{jwt_token}");
        let mut user = db_user.into_active_model();
        user.token = Set(Some(jwt_token));

        let saved_user = user
            .save(&db_conn)
            .await
            .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(ResponseAddUser {
            user_id: saved_user.id.unwrap(),
            username: saved_user.username.unwrap(),
            email: "xyz@gmail.com".to_owned(),
            token: saved_user.token.unwrap(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
pub async fn logout(
    State(db_conn): State<DatabaseConnection>,
    //TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Extension(user): Extension<UserModel>,
) -> Result<(), StatusCode> {
    let mut user = user.into_active_model();
    dbg!(&user);
    user.token = Set(None);
    let _saved_user = user
        .save(&db_conn)
        .await
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

// Serialize for output json
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
//------------------== Rest Read
#[derive(Serialize, Debug)]
pub struct ResponseTask {
    pub task_id: i32,
    pub title: String,
    pub priority: Option<String>,
    pub description: Option<String>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
    pub user_id: Option<i32>,
} //Find the field types from task: Option<Model>, then put them into the type fields inside this Output struct above; THEN add chronos with serde feature to serialize the output!

//curl localhost:3000/user/9
pub async fn get_task_by_id(
    State(db_conn): State<DatabaseConnection>,
    Path(task_id): Path<i32>,
) -> Result<Json<ResponseTask>, StatusCode> {
    let task = Tasks::find_by_id(task_id)
        .filter(tasks::Column::DeletedAt.is_null())
        .one(&db_conn)
        .await
        .map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR)?;

    dbg!(&task);

    if let Some(task) = task {
        Ok(Json(ResponseTask {
            task_id,
            title: task.title,
            priority: task.priority,
            description: task.description,
            deleted_at: task.deleted_at,
            user_id: task.user_id,
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Deserialize)]
pub struct GetTasksParams {
    pub task_id: Option<i32>,
    pub title: Option<String>,
    pub priority: Option<String>,
}
pub async fn get_tasks_all(
    State(db_conn): State<DatabaseConnection>,
    Query(query_params): Query<GetTasksParams>,
) -> Result<Json<Vec<ResponseTask>>, StatusCode> {
    let mut condition = Condition::all();
    if let Some(priority) = query_params.priority {
        dbg!(&priority);
        condition = if priority.is_empty() {
            condition.add(tasks::Column::Priority.is_null())
        } else {
            condition.add(tasks::Column::Priority.eq(priority))
        };
    }
    if let Some(title) = query_params.title {
        dbg!(&title);
        condition = if title.is_empty() {
            condition.add(tasks::Column::Title.is_null())
        } else {
            condition.add(tasks::Column::Title.eq(title))
        };
    }

    let tasks = Tasks::find()
        .filter(condition)
        .filter(tasks::Column::DeletedAt.is_null())
        .all(&db_conn)
        .await
        .map_err(|_error| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_iter()
        .map(|task| ResponseTask {
            task_id: task.id,
            title: task.title,
            priority: task.priority,
            description: task.description,
            deleted_at: task.deleted_at,
            user_id: task.user_id,
        })
        .collect();
    Ok(Json(tasks))
    //dbg!(&tasks);
}
//------------------== Rest Create(Add)
// Deserialize for input json, Debug for terminal print
#[derive(Deserialize, Debug)]
pub struct AddTask {
    pub title: String,
    pub priority: Option<String>,
    pub description: Option<String>,
}
#[derive(Serialize, Debug)]
pub struct ResponseAddTask {
    pub title: String,
    pub priority: Option<String>,
    pub description: Option<String>,
}
pub async fn add_task(
    State(db_conn): State<DatabaseConnection>,
    Extension(user): Extension<UserModel>,
    Json(json): Json<AddTask>,
    //TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    //auth: TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ResponseAddTask>, StatusCode> {
    let user = user.into_active_model();

    let new_task = tasks::ActiveModel {
        title: Set(json.title),
        priority: Set(json.priority),
        description: Set(json.description),
        user_id: Set(Some(user.id.unwrap())),
        ..Default::default()
    };
    let saved_task = new_task
        .save(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    dbg!(&saved_task);
    Ok(Json(ResponseAddTask {
        title: saved_task.title.unwrap(),
        priority: saved_task.priority.unwrap(),
        description: saved_task.description.unwrap(),
    }))
}
//------------------== Rest Put(Replace or Atomic update)
//PUT replacs the entire entity(overwrite any missing fields to null), while PATCH only updates the fields that you give it.
#[derive(Deserialize, Debug)]
pub struct ReplaceTask {
    pub id: Option<i32>,
    pub priority: Option<String>,
    pub title: String,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub description: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub user_id: Option<i32>,
    pub is_default: Option<bool>,
} //copied from entities/tasks.rs, change id to option so we keep the original id the same. Leave the rest unchange according to the DB settings
pub async fn replace_task(
    State(db_conn): State<DatabaseConnection>,
    Path(task_id): Path<i32>,
    Json(json): Json<ReplaceTask>,
) -> Result<String, StatusCode> {
    let replacing_task = tasks::ActiveModel {
        id: Set(task_id),
        priority: Set(json.priority),
        title: Set(json.title),
        completed_at: Set(json.completed_at),
        description: Set(json.description),
        deleted_at: Set(json.deleted_at),
        user_id: Set(json.user_id),
        is_default: Set(json.is_default),
    };
    Tasks::update(replacing_task)
        .filter(tasks::Column::Id.eq(task_id))
        .exec(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok("ok".to_owned())
}
//------------------== Rest Patch
#[derive(Deserialize, Debug)]
pub struct UpdatePartialTask {
    //Should not use serde_with with single option!!!
    pub id: Option<i32>,
    #[serde(
        default,// for deserialization
        skip_serializing_if = "Option::is_none",//serialization
        with = "::serde_with::rust::double_option",
    )]
    pub priority: Option<Option<String>>,
    //Should not be null, so do not add serde_with with double option macro here!!!
    pub title: Option<String>,
    #[serde(
        default,// for deserialization
        skip_serializing_if = "Option::is_none",//serialization
        with = "::serde_with::rust::double_option",
    )]
    pub description: Option<Option<String>>,
} // remove user_id, completed_at, deleted_at and is_default so those cannot be set!
pub async fn update_partial_task(
    State(db_conn): State<DatabaseConnection>,
    Path(task_id): Path<i32>,
    Json(json): Json<UpdatePartialTask>,
) -> Result<String, StatusCode> {
    let mut existing_task = if let Some(task) = Tasks::find_by_id(task_id)
        .one(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        task.into_active_model() //need active model
    } else {
        return Err(StatusCode::NOT_FOUND);
    };

    //if the priority field is set, even it is set to null
    if let Some(priority) = json.priority {
        existing_task.priority = Set(priority);
    }
    if let Some(description) = json.description {
        existing_task.description = Set(description);
    }
    if let Some(title) = json.title {
        existing_task.title = Set(title); //single option
    }

    dbg!(&existing_task);
    Tasks::update(existing_task)
        .filter(tasks::Column::Id.eq(task_id))
        .exec(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok("ok".to_owned())
}

//------------------== Rest Delete
#[derive(Deserialize, Debug)]
pub struct QueryParamsDelete {
    is_soft: bool,
}
pub async fn delete_task(
    State(db_conn): State<DatabaseConnection>,
    Path(task_id): Path<i32>,
    Query(query_params): Query<QueryParamsDelete>,
) -> Result<String, StatusCode> {
    let mut existing_task = if let Some(task) = Tasks::find_by_id(task_id)
        .one(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        task.into_active_model() //need active model
    } else {
        return Err(StatusCode::NOT_FOUND);
    };
    dbg!(&existing_task);
    if query_params.is_soft {
        dbg!("do soft delete"); //Note: soft delete can be recovered if you do a partial update and set deleted_at to null!
        let now = chrono::Utc::now();
        existing_task.deleted_at = Set(Some(now.into()));

        Tasks::update(existing_task)
            .filter(tasks::Column::Id.eq(task_id))
            .exec(&db_conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok("ok".to_owned())
    } else {
        dbg!("do hard delete");
        let delete_result = Tasks::delete(existing_task)
            .exec(&db_conn)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        if delete_result.rows_affected != 1 {
            return Err(StatusCode::EXPECTATION_FAILED);
        }
        Ok("ok".to_owned())
        /*
        if you do not want to check if the item exists:
        # Use delete_by_id
        Tasks::delete_by_id(task_id)
        .exec(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        # Use delete_many + filter ...
        Tasks::delete_many()
        .filter(tasks::Column::Id.eq(task_id))
        .exec(&db_conn)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        */
    }
}
#[derive(Serialize, Debug)]
pub struct RespBlockchain {
    pub num1: Option<String>,
    pub num2: Option<String>,
    pub address: Option<String>,
    pub txn_hash: Option<String>,
    pub error: Option<String>,
}
impl Default for RespBlockchain {
    fn default() -> Self {
        Self {
            num1: None,
            num2: None,
            address: None,
            txn_hash: None,
            error: None,
        }
    }
}
#[derive(Deserialize, Debug)]
pub struct ReqBlockchain {
    pub num1: Option<f64>,
    pub num2: Option<f64>,
    pub addr1: Option<String>,
    pub addr2: Option<String>,
}
pub async fn eth_local_txn(
    State(_db_conn): State<DatabaseConnection>,
    Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("eth_local_txn");
    dbg!(&json);

    let _txn_result = ethereum_local_txn().await.map_err(|_e| "err".to_owned())?;
    Ok(Json(RespBlockchain {
        ..Default::default()
    }))
}
pub async fn eth_deploy_contract(
    State(_db_conn): State<DatabaseConnection>,
    Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("eth_deploy_contract");
    dbg!(&json);

    let _txn_result = compile_deploy_contract()
        .await
        .map_err(|_e| "err".to_owned())?;
    Ok(Json(RespBlockchain {
        ..Default::default()
    }))
}
pub async fn eth_live_read(
    State(_db_conn): State<DatabaseConnection>,
    Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("eth_live_read");
    dbg!(&json);

    let (bal0, bal1) = ethereum_live_read().await.map_err(|e| e.to_string())?;
    Ok(Json(RespBlockchain {
        num1: Some(bal0),
        num2: Some(bal1),
        ..Default::default()
    }))
}
pub async fn eth_live_write(
    State(_db_conn): State<DatabaseConnection>,
    Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("eth_live_write");
    dbg!(&json);
    let amount = json.num1.ok_or_else(|| "num1 missing".to_owned())?;

    let (txn_hash, balance1) = ethereum_live_write(amount)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(RespBlockchain {
        num1: Some(balance1),
        txn_hash: Some(txn_hash),
        ..Default::default()
    }))
}
pub async fn eth_send_ether(
    State(_db_conn): State<DatabaseConnection>,
    Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("eth_send_ether");
    dbg!(&json);
    let amount = json.num1.ok_or_else(|| "num1 missing".to_owned())?;
    let addr1 = json.addr1.ok_or_else(|| "addr1 missing".to_owned())?;

    let (txn_hash, balance1) = ethereum_send_ether(addr1, amount)
        .await
        .map_err(|e| e.to_string())?;
    Ok(Json(RespBlockchain {
        num1: Some(balance1),
        txn_hash: Some(txn_hash),
        ..Default::default()
    }))
}
pub async fn chainlink_prices(
    State(_db_conn): State<DatabaseConnection>,
    //Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("get_ether_price");
    let (btc_price, eth_price) = get_chainlink_prices().await.map_err(|e| e.to_string())?;
    Ok(Json(RespBlockchain {
        num1: Some(btc_price),
        num2: Some(eth_price),
        ..Default::default()
    }))
}
pub async fn run_thread(
    State(_db_conn): State<DatabaseConnection>,
    Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    println!("run_thread");
    dbg!(&json);
    dbg!(json);
    let main1 = 1;
    let (tx, rx) = mpsc::channel::<u64>();
    let handle = thread::spawn(move || {
        for i in 1..5 {
            println!("inside thread. i = {}, main1 = {}", i, main1);
            thread::sleep(Duration::from_millis(1));
        }
        tx.send(1000).map_err(|err| {
            println!("err1: {err}");
            "tx.send() failed".to_owned()
        })
    });
    handle
        .join()
        .map_err(|_e| "handle.join() failed".to_owned())??;

    println!("main thread continue after waiting for the thread");
    let _out = rx.recv().map_err(|_e| "rx.recv() failed".to_owned())?;
    Ok(Json(RespBlockchain {
        ..Default::default()
    }))
}

use reqwest::header::USER_AGENT;
#[derive(Deserialize, Debug)]
pub struct Item {
    pub login: String,
    pub id: u32,
}
//curl localhost:3000/make_get_request
pub async fn make_get_request(
    State(_db_conn): State<DatabaseConnection>,
    //Path(task_id): Path<i32>,
) -> Result<Json<RespBlockchain>, String> {
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/stargazers",
        owner = "rust-lang-nursery",
        repo = "rust-cookbook"
    );
    println!("request_url: {}", request_url);

    let client = reqwest::Client::new();
    let response = client
        .get(&request_url)
        .header(USER_AGENT, "demo")
        .send()
        .await
        .map_err(|_| "err@send()".to_owned())?;

    let users: Vec<Item> = response.json().await.map_err(|_| "err@json()".to_owned())?;
    println!("users: {:?}", users);
    /*let resp = reqwest::get("https://httpbin.org/ip")
            .await?
            .json::<HashMap<String, String>>()
            .await?;
        println!("{:#?}", resp);
    */
    Ok(Json(RespBlockchain {
        ..Default::default()
    }))
}

//curl localhost:3000/make_get_request
pub async fn make_post_request(
    State(_db_conn): State<DatabaseConnection>,
    //Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    // This will POST a body of `{"lang":"rust","body":"json"}`
    let mut map = HashMap::new();
    map.insert("lang", "rust");
    map.insert("body", "json");

    let client = reqwest::Client::new();
    let res = client
        .post("http://httpbin.org/post")
        .json(&map)
        .send()
        .await
        .map_err(|_| "err@send()".to_owned())?;
    println!("res: {:?}", res);
    Ok(Json(RespBlockchain {
        ..Default::default()
    }))
}

use std::fs::File;
use std::io::copy;
use tempfile::Builder;
/*use error_chain::error_chain;
error_chain! {
  foreign_links {
      Io(std::io::Error);
      HttpRequest(reqwest::Error);
  }
}*/

pub async fn download_file(
    State(_db_conn): State<DatabaseConnection>,
    //Json(json): Json<ReqBlockchain>,
) -> Result<Json<RespBlockchain>, String> {
    let tmp_dir = Builder::new()
        .prefix("example")
        .tempdir()
        .map_err(|_| "err@tempdir()".to_owned())?;
    let target = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    println!("download_file 1");
    let response = reqwest::get(target)
        .await
        .map_err(|_| "err@reqwest.get()".to_owned())?;
    println!("download_file 2. response:{:?}", &response);

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| segments.last())
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");
        println!("file to download: '{}'", fname);

        let fname = tmp_dir.path().join(fname);
        println!("will be located at: '{:?}'", fname);
        File::create(fname)
    }
    .map_err(|_| "err@File::create()".to_owned())?;
    println!("download_file 3. dest:{:?}", &dest);

    let content = response
        .text()
        .await
        .map_err(|_| "err@response.text()".to_owned())?;
    println!("download_file 4");
    //println!("download_file 4. content:{}", &content);

    let out = copy(&mut content.as_bytes(), &mut dest).map_err(|_| "err@copy())".to_owned())?;
    println!("download_file 5. out:{}", out);

    Ok(Json(RespBlockchain {
        num1: Some(out.to_string()),
        ..Default::default()
    }))
}
