use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, required_u64};
use serde_json::Value;

/// Tool definitions for query result tools.
pub fn definitions() -> Vec<Value> {
    vec![serde_json::json!({
        "name": "get_query_result",
        "description": "Get the latest cached result of a query",
        "inputSchema": {
            "type": "object",
            "properties": {
                "id": {
                    "type": "integer",
                    "description": "Query ID"
                }
            },
            "required": ["id"]
        }
    })]
}

/// Get the latest cached result of a query.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/queries/{id}/results.json")).await?;
    Ok(format_tool_result(&data))
}
