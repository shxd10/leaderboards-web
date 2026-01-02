use crate::{
    models::{User, UserLoginData},
    response::{ApiError, ApiResponse, BasicMessage},
    routes::{Claims, AuthenticatedUser, JWT_SECRET}
};
use axum::{extract::{Path, State, Json, Extension}};
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;
use argon2::{password_hash::{rand_core::OsRng, SaltString}, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use jsonwebtoken::{EncodingKey, Header, encode};
use chrono::Utc;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub id: Option<i64>,
    pub username: Option<String>,
}

#[axum::debug_handler]
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<ApiResponse<User>, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let hash_password = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .to_string();

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO user (username, password_hash) VALUES (?, ?) RETURNING id, username, created_at"
    )
    .bind(&payload.username)
    .bind(&hash_password)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    
    Ok(ApiResponse::JsonData(user))
}

pub async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<ApiResponse<LoginResponse>, ApiError> {
    let user_login_data = sqlx::query_as::<_, UserLoginData>(
        "SELECT id, password_hash FROM user WHERE username = ?"
    )
    .bind(&payload.username)
    .fetch_optional(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let user_login_data = match user_login_data {
        Some(r) => r,
        None => return Err(ApiError::Unauthorized("Invalid username or password".into())),
    };

    let parsed_hash = PasswordHash::new(&user_login_data.password_hash)
        .map_err(|_| ApiError::InternalServerError("Invalid password hash".to_string()))?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| ApiError::Unauthorized("Invalid username or password".to_string()))?;

    // create JWT
    let expiration = Utc::now().timestamp() as usize + 24*60*60;
    let claims = Claims { sub: user_login_data.id, exp: expiration };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(&JWT_SECRET))
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(LoginResponse { token }))
}

pub async fn current_user_info(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> Result<ApiResponse<User>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, created_at FROM user WHERE id = ?"
    )
    .bind(auth.id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::Unauthorized("User not found".into()))?;

    Ok(ApiResponse::JsonData(user))
}

pub async fn get_user(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>
) -> Result<ApiResponse<User>, ApiError> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, username, created_at FROM user WHERE id = ?"
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(user))
}

pub async fn patch_user(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<ApiResponse<User>, ApiError> {

    let user = sqlx::query_as::<_, User>(
        "UPDATE user
        SET
            id = COALESCE(?, id),
            username = COALESCE(?, username)
        WHERE id = ?
        RETURNING id, username, created_at"
    )
    .bind(payload.id) // new id
    .bind(payload.username) // new username
    .bind(id) // current id
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(user))
}

pub async fn delete_user(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>
) -> Result<ApiResponse<BasicMessage>, ApiError> {
    sqlx::query(
        "DELETE FROM user WHERE id = ?"
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(BasicMessage { msg: "User deleted".to_string() }))
}

pub async fn get_users(
    State(pool): State<SqlitePool>
) -> Result<ApiResponse<Vec<User>>, ApiError> {
    let users: Vec<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, created_at FROM user"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(users))
}
