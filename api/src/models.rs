use sqlx::FromRow;
use serde::Serialize;

// DB struct models

// exposed user
#[derive(Debug, Clone, FromRow, Serialize)]
pub struct User {
    pub id: i64,
    pub role: String,
    pub username: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct UserLoginData {
    pub id: i64,
    pub role: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Category {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Cup {
    pub id: i64,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Track {
    pub id: i64,
    pub cup_id: i64,
    pub slug: String,
    pub name: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Record {
    pub id: i64,
    pub user_id: i64,
    pub track_id: i64,
    pub category_id: i64,
    pub flap: bool,
    pub lap1: Option<i64>,
    pub lap2: Option<i64>,
    pub lap3: Option<i64>,
    pub time_ms: i64,
    pub proof: String,
    pub submitted_at: String,
}