use serde::Deserialize;
use struct_iterable::Iterable;

#[derive(Deserialize, Debug)]
pub struct CreateNodePayload {
    pub owner_id: String,
    pub name: String,
    pub ip: Option<String>,
    pub port: i32,
}

#[derive(Deserialize, Debug, Iterable)]
pub struct UpdateNodePayload {
    pub owner_id: Option<String>,
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
}
