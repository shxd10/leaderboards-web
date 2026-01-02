use axum::{Router, routing::{get, post, delete, put, patch}};
use crate::response::{ApiError, ApiResponse, BasicMessage};
use sqlx::SqlitePool;
use axum::{middleware::{Next, from_fn}, http::Request, response::Response, body::Body};
use jsonwebtoken::{DecodingKey, Validation, decode, TokenData};
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;

pub mod users;

use users::{get_user, get_users, create_user, patch_user, delete_user, login, current_user_info};

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/signup", post(create_user))
                .route("/login", post(login))
                .route("/you", get(current_user_info).layer(from_fn(auth_middleware)))
                .route("/user/{id}", get(get_user).patch(patch_user).delete(delete_user))
                .route("/users", get(get_users))
                .with_state(pool)
                .fallback(not_found)
        )
}

async fn not_found() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Err(ApiError::NotFound("Route not found".to_string()))
}

// JWT shit

pub static JWT_SECRET: Lazy<Vec<u8>> = Lazy::new(|| {
    std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default_secret".to_string())
        .into_bytes()
});

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,    // user id
    pub exp: usize,  // expiration timestamp (epoch)
}

pub async fn auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing token".into()))?;

    // verify token
    let token_data: TokenData<Claims> = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&JWT_SECRET),
        &Validation::default()
    ).map_err(|_| ApiError::Unauthorized("Invalid token".into()))?;

    req.extensions_mut().insert(AuthenticatedUser { id: token_data.claims.sub });

    Ok(next.run(req).await)
}
