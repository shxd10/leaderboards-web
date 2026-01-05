use crate::{
    models::{Cup, Category, Record, Track},
    response::{ApiError, ApiResponse},
    routes::AuthenticatedUser,
};
use axum::{extract::{Path, State, Json, Extension}};
use serde::{Serialize, Deserialize};
use sqlx::SqlitePool;

#[derive(Serialize)]
pub struct TrackLeaderboard {
    pub track: Track,
    pub records: Vec<Record>,
}

#[derive(Deserialize)]
pub struct NewRecordRequest {
    pub track_slug: String,
    pub category_slug: String,
    pub flap: bool,
    pub lap1: Option<i64>,
    pub lap2: Option<i64>,
    pub lap3: Option<i64>,
    pub time_ms: i64,
    pub proof: Option<String>,
}

////

async fn get_category(
    category_slug: &str,
    pool: &SqlitePool
) -> Result<Category, ApiError> {
    let category: Category = sqlx::query_as::<_, Category>(
        "SELECT id, slug, name FROM category WHERE slug = ?"
    )
    .bind(category_slug)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::NotFound("Category not found".to_string()))?;

    Ok(category)
}

async fn get_cup(
    cup_slug: &str,
    pool: &SqlitePool
) -> Result<Cup, ApiError> {
    let cup: Cup = sqlx::query_as::<_, Cup>(
        "SELECT id, slug, name FROM cup WHERE slug = ?"
    )
    .bind(cup_slug)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::NotFound("Cup not found".to_string()))?;

    Ok(cup)
}

async fn get_track(
    track_slug: &str,
    pool: &SqlitePool
) -> Result<Track, ApiError> {
    let track: Track = sqlx::query_as::<_, Track>(
        "SELECT id, cup_id, slug, name FROM track WHERE slug = ?"
    )
    .bind(track_slug)
    .fetch_one(pool)
    .await
    .map_err(|_| ApiError::NotFound("Track not found".to_string()))?;

    Ok(track)
}

async fn get_tracks_in_cup(
    cup_id: i64,
    pool: &SqlitePool
) -> Result<Vec<Track>, ApiError> {
    let tracks: Vec<Track> = sqlx::query_as::<_, Track>(
        "SELECT id, cup_id, slug, name FROM track WHERE cup_id = ?"
    )
    .bind(cup_id)
    .fetch_all(pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(tracks)
}

////

pub async fn get_cup_leaderboard(
    Path((flap, category_slug, cup_slug)): Path<(String, String, String)>,
    State(pool): State<SqlitePool>
) -> Result<ApiResponse<Vec<TrackLeaderboard>>, ApiError> {
    let flap = match flap.as_str() {
        "flap" => true,
        "3lap" => false,
        _ => return Err(ApiError::BadRequest("Invalid record type".to_string())),
    };
    let category: Category = get_category(&category_slug, &pool).await?;
    let cup: Cup = get_cup(&cup_slug, &pool).await?;
    let tracks: Vec<Track> = get_tracks_in_cup(cup.id, &pool).await?;

    let mut leaderboards: Vec<TrackLeaderboard> = Vec::new();

    for track in tracks {
        let records: Vec<Record> = sqlx::query_as::<_, Record>(
            "SELECT * FROM record WHERE track_id = ? AND category_id = ? AND flap = ? ORDER BY time_ms ASC"
        )
        .bind(track.id)
        .bind(category.id)
        .bind(flap)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

        leaderboards.push(TrackLeaderboard { track, records });
    }

    Ok(ApiResponse::JsonData(leaderboards))
}

#[axum::debug_handler]
pub async fn post_record(
    State(pool): State<SqlitePool>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(payload): Json<NewRecordRequest>,
) -> Result<ApiResponse<Record>, ApiError> {
    let track = get_track(&payload.track_slug, &pool).await?;
    let category = get_category(&payload.category_slug, &pool).await?;

    let record: Record = sqlx::query_as::<_, Record>(
        "INSERT INTO record (user_id, track_id, category_id, flap, lap1, lap2, lap3, time_ms, proof)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *"
    )
    .bind(auth.id)
    .bind(track.id)
    .bind(category.id)
    .bind(payload.flap)
    .bind(payload.lap1)
    .bind(payload.lap2)
    .bind(payload.lap3)
    .bind(payload.time_ms)
    .bind(payload.proof)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(record))
}

pub async fn post_record_admin(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(payload): Json<NewRecordRequest>,
) -> Result<ApiResponse<Record>, ApiError> {
    let track = get_track(&payload.track_slug, &pool).await?;
    let category = get_category(&payload.category_slug, &pool).await?;

    let record: Record = sqlx::query_as::<_, Record>(
        "INSERT INTO record (user_id, track_id, category_id, flap, lap1, lap2, lap3, time_ms, proof)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *"
    )
    .bind(id)
    .bind(track.id)
    .bind(category.id)
    .bind(payload.flap)
    .bind(payload.lap1)
    .bind(payload.lap2)
    .bind(payload.lap3)
    .bind(payload.time_ms)
    .bind(payload.proof)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    Ok(ApiResponse::JsonData(record))
}