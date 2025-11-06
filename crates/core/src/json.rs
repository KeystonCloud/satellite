use serde::Serialize;

#[derive(Serialize)]
pub struct SimpleJsonResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct DataJsonResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct ErrorJsonResponse {
    pub error: String,
}
