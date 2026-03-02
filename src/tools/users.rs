use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_u64, required_u64};
use serde_json::Value;

/// Tool definitions for user tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_users",
            "description": "List users with optional pagination",
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
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "get_user",
            "description": "Get details of a specific user by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "User ID"
                    }
                },
                "required": ["id"]
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
    ]
}

/// List users with optional pagination.
pub async fn list(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!("/users?page={page}&page_size={page_size}"))
        .await?;
    Ok(format_tool_result(&data))
}

/// Get a specific user by ID.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/users/{id}")).await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn users_definitions_count() {
        assert_eq!(definitions().len(), 2);
    }

    #[test]
    fn list_users_definition_no_required() {
        let defs = definitions();
        let list_def = defs.iter().find(|d| d["name"] == "list_users").unwrap();
        let required = list_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn get_user_definition_required_fields() {
        let defs = definitions();
        let get_def = defs.iter().find(|d| d["name"] == "get_user").unwrap();
        let required = get_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }
}
