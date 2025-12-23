use crate::response::{ApiError, ApiResponse};
use axum::{extract::Path};
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub id: u32,
    pub username: String,
}

pub async fn get_users() -> Result<ApiResponse<Vec<User>>, ApiError> {
    let users = vec![
        User {
            id: 1,
            username: "user1".to_string(),
        },
        User {
            id: 2,
            username: "user2".to_string(),
        },
    ];

    Ok(ApiResponse::JsonData(users))
}

// soon DB
pub async fn get_user(Path(id): Path<u32>) -> Result<ApiResponse<User>, ApiError> {
    let user = User {
        id,
        username: format!("user{}", id),
    };

    Ok(ApiResponse::JsonData(user))
}
