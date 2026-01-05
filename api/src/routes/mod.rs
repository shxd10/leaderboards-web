use axum::{Router, routing::{get, post}};
use crate::response::{ApiError, ApiResponse, BasicMessage};
use sqlx::SqlitePool;
use axum::{middleware::{Next, from_fn}, http::Request, response::Response, body::Body};
use jsonwebtoken::{DecodingKey, Validation, decode, TokenData};
use serde::{Serialize, Deserialize};
use once_cell::sync::Lazy;

pub mod users;
pub mod records;

use users::*;
use records::*;

// protected as in, the user needs to be logged in to send requests
fn protected_routes() -> Router<SqlitePool> {
    Router::new()
        .route("/me", get(get_current_user).patch(patch_current_user).delete(delete_current_user))
        .route("/new", post(post_record))
        .merge(admin_routes())
        .layer(from_fn(auth_middleware))
}

fn admin_routes() -> Router<SqlitePool> {
    Router::new()
        .nest(
            "/admin",
            Router::new()
                .route("/user/{id}", get(get_user).patch(patch_user).delete(delete_user).post(post_record_admin))
                .layer(from_fn(admin_only))
        )
}

pub fn router(pool: SqlitePool) -> Router {
    Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/signup", post(create_user))
                .route("/login", post(login))
                .route("/user/{id}", get(get_user))
                .route("/users", get(get_users))

                .route("/{record_type}/{category}/{cup}", get(get_cup_leaderboard))

                .merge(protected_routes())

                .with_state(pool)
                .fallback(not_found)
        )
}

async fn not_found() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Err(ApiError::NotFound("Route not found".to_string()))
}

// JWT shit that i don't even understand

pub static JWT_SECRET: Lazy<Vec<u8>> = Lazy::new(|| {
    std::env::var("JWT_SECRET")
        .expect("JWT_SECRET must be set")
        .into_bytes()
});

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub id: i64,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,    // user id
    pub role: String,
    pub exp: usize,  // expiration timestamp (epoch)
}

pub async fn auth_middleware(
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    // Requests with authorization will include an "Authorization" header with
    // value "Bearer <token>"
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Missing token".into()))?;

    // verify the token
    let token_data: TokenData<Claims> = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&JWT_SECRET),
        &Validation::default()
    ).map_err(|_| ApiError::Unauthorized("Invalid token".into()))?;

    req.extensions_mut().insert(AuthenticatedUser {
        id: token_data.claims.sub,
        role: token_data.claims.role,
    });

    Ok(next.run(req).await)
}

pub async fn admin_only(
    req: Request<Body>,
    next: Next,
) -> Result<Response, ApiError> {
    let auth = req
        .extensions()
        .get::<AuthenticatedUser>()
        .ok_or_else(|| ApiError::Unauthorized("Not authenticated".into()))?;

    if auth.role != "admin" {
        return Err(ApiError::Forbidden("Admin access required".into()));
    }

    Ok(next.run(req).await)
}
