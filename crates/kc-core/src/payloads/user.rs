use serde::Deserialize;
use struct_iterable::Iterable;

#[derive(Deserialize, Debug)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone, Iterable)]
pub struct UpdateUserPayload {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub new_password: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}
