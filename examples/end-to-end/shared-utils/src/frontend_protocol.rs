use crate::NodeConfig;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeMetaDataResponse {
    pub node_id: Uuid,
    pub last_seen: chrono::DateTime<Utc>,
    pub config: NodeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeConfigRequest {
    pub node_id: Uuid,
    pub config: NodeConfig,
}
