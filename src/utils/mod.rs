use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub fn hash_password(password: String) -> Result<String, StatusCode> {
    //Set bcrypt hash cost to 14 or above to ensure enought time cost against hackers
    let hashcost = dotenvy::var("HASHCOST").expect("HASHCOST not found in .env");
    let hashcostu32 = hashcost.parse::<u32>().expect("HASHCOST not u32");
    bcrypt::hash(password, hashcostu32).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
    //cost should be 1 ~ 31
    //bcrypt(cost, salt, password)
}
pub fn verify_password(password: String, hash: &str) -> Result<bool, StatusCode> {
    bcrypt::verify(password, hash).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)
}

//https://crates.io/crates/jsonwebtoken
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    //aud: String, // Optional. Audience
    exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    iat: usize, // Optional. Issued at (as UTC timestamp)
                //iss: String, // Optional. Issuer
                //nbf: usize,  // Optional. Not Before (as UTC timestamp)
                //sub: String, // Optional. Subject (whom token refers to)
}
pub fn make_jwt() -> Result<String, StatusCode> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let duration = Duration::seconds(30);
    let exp = (now + duration).timestamp() as usize;
    let claim = Claims { exp, iat };

    let secret = dotenvy::var("JWT_SECRET").expect("secret not found in .env");

    let token = encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR);
    token
}

pub fn verify_jwt(token: &str) -> Result<bool, AppError> {
    //verify(password, hash)
    let secret = dotenvy::var("JWT_SECRET").expect("secret key not found in .env");
    let key = DecodingKey::from_secret(secret.as_bytes());
    let validation = &Validation::new(Algorithm::HS256);
    let _result =
        decode::<Claims>(token, &key, validation).map_err(|error| match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                println!("verify_jwt: token expired");
                AppError::new(StatusCode::UNAUTHORIZED, "session expired. error 010")
            }
            _ => {
                println!("verify_jwt: token invalid");
                AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "error 011")
            }
        })?;
    Ok(true)
}

pub struct AppError {
    code: StatusCode,
    message: String,
}
impl AppError {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
//Axum's trait
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            self.code,
            Json(ResponseMessage {
                message: self.message,
            }),
        )
            .into_response()
    }
}
#[derive(Debug, Serialize)]
struct ResponseMessage {
    message: String,
}
