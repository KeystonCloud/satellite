use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct AppInfo {
    pub name: String,
    pub current_cid: String,
    pub key_name: String,
    pub ipns_name: String,
}

pub type AppRegistry = Arc<Mutex<HashMap<String, AppInfo>>>;
