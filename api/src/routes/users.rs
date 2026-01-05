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
    pub username: Option<String>,
    pub password: Option<String>,
}

////

#[axum::debug_handler]
pub async fn create_user(
    State(pool): State<SqlitePool>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<ApiResponse<User>, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?
        .to_string();

    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO user (username, password_hash) VALUES (?, ?) RETURNING id, username, role, created_at"
    )
    .bind(&payload.username)
    .bind(&password_hash)
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
        "SELECT id, role, password_hash FROM user WHERE username = ?"
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
    let claims = Claims { sub: user_login_data.id, role: user_login_data.role, exp: expiration };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(&JWT_SECRET))
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(LoginResponse { token }))
}

////

pub async fn get_user(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>
) -> Result<ApiResponse<User>, ApiError> {
    let user: User = sqlx::query_as::<_, User>(
        "SELECT id, role, username, created_at FROM user WHERE id = ?"
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
    let password_hash: Option<String> = if let Some(password) = payload.password {
        let salt = SaltString::generate(&mut OsRng);
        Some(
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?
                .to_string()
        )
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE user
        SET
            username = COALESCE(?, username),
            password_hash = COALESCE(?, password_hash)
        WHERE id = ?
        RETURNING id, role, username, created_at
        "#
    )
    .bind(payload.username)
    .bind(password_hash)
    .bind(id)
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
        "SELECT id, role, username, created_at FROM user"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(users))
}

////

pub async fn get_current_user(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> Result<ApiResponse<User>, ApiError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, role, username, created_at FROM user WHERE id = ?"
    )
    .bind(auth.id)
    .fetch_one(&pool)
    .await
    .map_err(|_| ApiError::Unauthorized("User not found".into()))?;

    Ok(ApiResponse::JsonData(user))
}

pub async fn patch_current_user(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<ApiResponse<User>, ApiError> {

    let password_hash: Option<String> = if let Some(password) = payload.password {
        let salt = SaltString::generate(&mut OsRng);
        Some(
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map_err(|e| ApiError::InternalServerError(e.to_string()))?
                .to_string()
        )
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE user
        SET
            username = COALESCE(?, username),
            password_hash = COALESCE(?, password_hash)
        WHERE id = ?
        RETURNING id, role, username, created_at
        "#
    )
    .bind(payload.username)
    .bind(password_hash)
    .bind(auth.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(user))
}

pub async fn delete_current_user(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> Result<ApiResponse<BasicMessage>, ApiError> {
    sqlx::query(
        "DELETE FROM user WHERE id = ?"
    )
    .bind(auth.id)
    .execute(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(BasicMessage { msg: "User deleted".to_string() }))
}