use crate::NodeConfig;
use serde::{Deserialize, Serialize};

/// Used to communicate from the node Main/UI-thread to the data workers.
#[derive(Debug, Deserialize, Serialize)]
pub enum ToDataWorker {
    /// Apply this configuration to the worker.
    ApplyConfig {
        /// The configuration to apply.
        config: NodeConfig,
    },
    /// Data to process.
    Data {
        /// The data to process.
        data: Vec<i8>,
    },
}
