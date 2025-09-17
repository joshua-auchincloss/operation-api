use std::collections::HashMap;

#[derive(serde::Deserialize, Debug, PartialEq)]
pub struct RemoteConfig {
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}
