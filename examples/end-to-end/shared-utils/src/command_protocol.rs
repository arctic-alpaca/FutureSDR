use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

// SQLx does not compile for wasm32 (the socket2 crate in particular).
/// Used to mark what kind of data a node does or should produce. Also used in path parameter for node and frontend.
#[derive(Debug, Deserialize, Serialize, Hash, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::Type))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    sqlx(type_name = "data_type_marker", rename_all = "lowercase")
)]
pub enum DataTypeMarker {
    Fft,
    ZigBee,
}

impl Display for DataTypeMarker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DataTypeMarker::Fft => {
                write!(f, "FFT")
            }
            DataTypeMarker::ZigBee => {
                write!(f, "ZigBee")
            }
        }
    }
}

/// SDR parameters used to set the configuration of a node.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NodeConfig {
    // default node config taken from https://github.com/bastibl/webusb-libusb/blob/works/example/hackrf_open.cc#L161
    // steps and ranges taken from https://hackrf.readthedocs.io/en/latest/faq.html#what-gain-controls-are-provided-by-hackrf
    // Sample rate and frequency taken from https://hackrf.readthedocs.io/en/latest/hackrf_one.html
    /// Set of workers to start
    pub data_types: HashSet<DataTypeMarker>,

    // default: 2480000000 (2,48GHz)
    // 1MHz to 6 GHz (1000000 - 6000000000)
    pub freq: u64,
    // default 1
    // on or off (0 or 1)
    pub amp: u8,
    // default: 32
    // 0-40 in steps of 8
    pub lna: u8,
    // default: 14
    // 0-62 in steps of 2
    pub vga: u8,
    // default: 4000000 (4 Msps)
    // 1 Msps to 20 Msps (million samples per second) (1000000 - 20000000)
    pub sample_rate: u64,
}

/// Used to communicate from the UI/main thread to the control worker.
#[derive(Debug, Deserialize, Serialize)]
pub enum ToCommandMsg {
    Initialize,
    AckConfig { config: NodeConfig },
    GetInitialConfig,
}

/// Used to communicate from the control worker to the UI/main thread.
#[derive(Debug, Deserialize, Serialize)]
pub enum FromCommandMsg {
    ReceivedConfig { config: NodeConfig },
    PrintToScreen { msg: String },
    Disconnected,
    Terminate { msg: String },
}

/// Used for internal communication of the control worker.
#[derive(Debug, Deserialize, Serialize)]
pub enum InternalCommandMsg {
    SendConfig { config: NodeConfig },
}

/// Used to communicate from the node to the backend.
#[derive(Debug, Deserialize, Serialize)]
pub enum NodeToBackend {
    RequestConfig,
    AckConfig { config: NodeConfig },
}

/// Used to communicate from the node to the backend.
#[derive(Debug, Deserialize, Serialize)]
pub enum BackendToNode {
    SendConfig { config: NodeConfig },
    Error { msg: String, terminate: bool },
}
