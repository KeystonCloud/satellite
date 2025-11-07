use serde::Deserialize;
use struct_iterable::Iterable;

#[derive(Deserialize, Debug)]
pub struct CreateAppPayload {
    pub team_id: String,
    pub name: String,
    pub key_name: String,
    pub ipns_name: String,
}

#[derive(Deserialize, Debug, Iterable)]
pub struct UpdateAppPayload {
    pub team_id: Option<String>,
    pub name: Option<String>,
}
