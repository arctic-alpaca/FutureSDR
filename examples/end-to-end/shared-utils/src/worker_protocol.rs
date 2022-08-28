use crate::NodeConfig;
use serde::{Deserialize, Serialize};

/// Used to communicate from the node to the backend.
#[derive(Debug, Deserialize, Serialize)]
pub enum ToWorker {
    ApplyConfig { config: NodeConfig },
    Data { data: Vec<i8> },
}
