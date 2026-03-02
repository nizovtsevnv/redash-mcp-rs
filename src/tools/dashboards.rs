use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_u64, required_string};
use serde_json::Value;

/// Tool definitions for dashboard tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_dashboards",
            "description": "List dashboards with optional pagination",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "page": {
                        "type": "integer",
                        "description": "Page number (default: 1)"
                    },
                    "page_size": {
                        "type": "integer",
                        "description": "Results per page (default: 25)"
                    }
                },
                "required": []
            }
        }),
        serde_json::json!({
            "name": "get_dashboard",
            "description": "Get a dashboard with its widgets by slug",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": {
                        "type": "string",
                        "description": "Dashboard slug"
                    }
                },
                "required": ["slug"]
            }
        }),
    ]
}

/// List dashboards with optional pagination.
pub async fn list(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!("/dashboards?page={page}&page_size={page_size}"))
        .await?;
    Ok(format_tool_result(&data))
}

/// Get a dashboard by slug.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let slug = required_string(args, "slug")?;
    let data = client.get(&format!("/dashboards/{slug}")).await?;
    Ok(format_tool_result(&data))
}
