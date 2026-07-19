use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub pack_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EntityReference {
    pub id: String,
    pub entity_id: String,
    pub asset_id: Option<String>,
    pub image_path: Option<String>,
    pub is_positive: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CreateEntityRequest {
    pub entity_type: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
}
