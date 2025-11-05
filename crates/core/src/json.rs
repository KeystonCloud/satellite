use serde::Serialize;

#[derive(Serialize)]
pub struct SimpleJsonResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct ModelJsonResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<String>,
}
