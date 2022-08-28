pub(crate) mod control;
pub(crate) mod data;

use crate::application::NodeId;
use std::str::FromStr;
use tower_cookies::Cookies;
use tracing::error;

fn extract_node_id_cookie(cookies: Cookies) -> anyhow::Result<NodeId> {
    if let Some(node_id_cookie) = cookies.get("node_id") {
        let node_id_uuid = uuid::Uuid::from_str(node_id_cookie.value())?;
        Ok(NodeId(node_id_uuid))
    } else {
        error!("No cookie \"node_id\"");
        anyhow::bail!("No cookie \"node_id\"");
    }
}
