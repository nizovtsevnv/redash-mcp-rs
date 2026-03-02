use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_u64, required_string, required_u64};
use serde_json::Value;

/// Tool definitions for query tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_queries",
            "description": "List saved queries with optional pagination",
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
            "name": "get_query",
            "description": "Get details of a specific query by ID, including the SQL",
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
        }),
        serde_json::json!({
            "name": "search_queries",
            "description": "Search queries by name or description",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "q": {
                        "type": "string",
                        "description": "Search term"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number (default: 1)"
                    },
                    "page_size": {
                        "type": "integer",
                        "description": "Results per page (default: 25)"
                    }
                },
                "required": ["q"]
            }
        }),
    ]
}

/// List saved queries with optional pagination.
pub async fn list(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!("/queries?page={page}&page_size={page_size}"))
        .await?;
    Ok(format_tool_result(&data))
}

/// Get a specific query by ID.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/queries/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Search queries by name or description.
pub async fn search(client: &RedashClient, args: &Value) -> Result<Value> {
    let q = required_string(args, "q")?;
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!(
            "/queries/search?q={}&page={page}&page_size={page_size}",
            urlencoded(&q)
        ))
        .await?;
    Ok(format_tool_result(&data))
}

/// Percent-encode a query string value.
fn urlencoded(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
