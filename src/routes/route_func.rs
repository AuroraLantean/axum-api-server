use axum::{
    async_trait,
    body::HttpBody,
    extract::{FromRequest, Path, Query},
    headers::UserAgent,
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

use crate::entities::{
    tasks::{self, Entity as Tasks},
    users::{self, Entity as Users},
};

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

pub async fn validate_struct_input(json: AddUser) -> impl IntoResponse {
    dbg!(json);
    (StatusCode::CREATED, "new user added".to_owned()).into_response()
}

pub async fn add_user(
    Extension(db_conn): Extension<DatabaseConnection>,
    //json: AddUser, //Must use this json format for validation!
    Json(json): Json<AddUser>,
) -> Result<Json<ResponseAddUser>, StatusCode> {
    dbg!(&json);
    let new_user = users::ActiveModel {
        username: Set(json.username),
        password: Set(json.password),
        token: Set(Some("abcdef123455676".to_owned())),
        ..Default::default()
    }
    .save(&db_conn)
    .await
    .map_err(|err| {
        dbg!("saving new user failed: {:?}", err);
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
#[derive(Deserialize, Debug, Validate)]
pub struct Login {
    pub username: String,
    #[validate(length(min = 8, message = "must have at least 8 characters"))]
    pub password: String,
} //Option field in input struct
pub async fn login(
    Extension(db_conn): Extension<DatabaseConnection>,
    Json(json): Json<Login>,
) -> Result<Json<ResponseAddUser>, StatusCode> {
    dbg!("input json", &json);
    let db_user = Users::find()
        .filter(users::Column::Username.eq(json.username))
        .one(&db_conn)
        .await
        .map_err(|err| {
            dbg!("finding user failed: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    dbg!("db_user", &db_user);

    if let Some(db_user) = db_user {
        let new_token = "1234567890".to_owned();
        let mut user = db_user.into_active_model();
        user.token = Set(Some(new_token));

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
} //Find the field types from task: Option<Model>, then put them into the type fields inside this Output struct above; THEN add chronos with serde feature to serialize the output!

//curl localhost:3000/user/9
pub async fn get_task_by_id(
    Extension(db_conn): Extension<DatabaseConnection>,
    Path(task_id): Path<i32>,
) -> Result<Json<ResponseTask>, StatusCode> {
    let task = Tasks::find_by_id(task_id)
        .filter(tasks::Column::DeletedAt.is_null())
        .one(&db_conn)
        .await
        .unwrap();
    dbg!(&task);

    if let Some(task) = task {
        Ok(Json(ResponseTask {
            task_id,
            title: task.title,
            priority: task.priority,
            description: task.description,
            deleted_at: task.deleted_at,
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
    Extension(db_conn): Extension<DatabaseConnection>,
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
    Extension(db_conn): Extension<DatabaseConnection>,
    Json(json): Json<AddTask>,
) -> Result<Json<ResponseAddTask>, StatusCode> {
    let new_task = tasks::ActiveModel {
        title: Set(json.title),
        priority: Set(json.priority),
        description: Set(json.description),
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
    Extension(db_conn): Extension<DatabaseConnection>,
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
    Extension(db_conn): Extension<DatabaseConnection>,
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
    Extension(db_conn): Extension<DatabaseConnection>,
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
