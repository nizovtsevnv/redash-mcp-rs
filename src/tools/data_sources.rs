use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, required_u64};
use serde_json::Value;

/// Tool definitions for data source tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_data_sources",
            "description": "List all available data sources in Redash",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
        serde_json::json!({
            "name": "get_data_source",
            "description": "Get details of a specific data source by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Data source ID"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "get_data_source_schema",
            "description": "Get the schema (tables and columns) of a data source. Essential for writing queries.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Data source ID"
                    }
                },
                "required": ["id"]
            }
        }),
    ]
}

/// List all data sources.
pub async fn list(client: &RedashClient) -> Result<Value> {
    let data = client.get("/data_sources").await?;
    Ok(format_tool_result(&data))
}

/// Get a specific data source by ID.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/data_sources/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Get the schema of a data source by ID.
pub async fn get_schema(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/data_sources/{id}/schema")).await?;
    Ok(format_tool_result(&data))
}
