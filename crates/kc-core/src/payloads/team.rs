use serde::Deserialize;
use struct_iterable::Iterable;

#[derive(Deserialize, Debug)]
pub struct CreateTeamPayload {
    pub name: String,
}

#[derive(Deserialize, Debug, Iterable)]
pub struct UpdateTeamPayload {
    pub name: Option<String>,
}
