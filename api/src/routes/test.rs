use crate::response::{ApiError, ApiResponse, BasicMessage};

pub async fn test() -> Result<ApiResponse<BasicMessage>, ApiError> {
    Ok(ApiResponse::JsonData(BasicMessage {
        msg: "test".to_string(),
    }))
}