use serde::Serialize;

#[derive(Serialize)]
pub struct SimpleJsonResponse {
    pub message: String,
}
