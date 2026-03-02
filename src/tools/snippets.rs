use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_string, required_string, required_u64};
use serde_json::Value;

/// Tool definitions for query snippet tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_query_snippets",
            "description": "List all query snippets",
            "inputSchema": {
                "type": "object",
                "properties": {},
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
            "name": "create_query_snippet",
            "description": "Create a new query snippet",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "trigger": {
                        "type": "string",
                        "description": "Trigger keyword for the snippet"
                    },
                    "description": {
                        "type": "string",
                        "description": "Snippet description"
                    },
                    "snippet": {
                        "type": "string",
                        "description": "SQL snippet content"
                    }
                },
                "required": ["trigger", "description", "snippet"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "get_query_snippet",
            "description": "Get a query snippet by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Snippet ID"
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
        serde_json::json!({
            "name": "update_query_snippet",
            "description": "Update an existing query snippet",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Snippet ID"
                    },
                    "trigger": {
                        "type": "string",
                        "description": "New trigger keyword"
                    },
                    "description": {
                        "type": "string",
                        "description": "New description"
                    },
                    "snippet": {
                        "type": "string",
                        "description": "New SQL snippet content"
                    }
                },
                "required": ["id"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "delete_query_snippet",
            "description": "Delete a query snippet by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Snippet ID"
                    }
                },
                "required": ["id"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
    ]
}

/// List all query snippets.
pub async fn list(client: &RedashClient) -> Result<Value> {
    let data = client.get("/query_snippets").await?;
    Ok(format_tool_result(&data))
}

/// Create a new query snippet.
pub async fn create(client: &RedashClient, args: &Value) -> Result<Value> {
    let trigger = required_string(args, "trigger")?;
    let description = required_string(args, "description")?;
    let snippet = required_string(args, "snippet")?;

    let body = serde_json::json!({
        "trigger": trigger,
        "description": description,
        "snippet": snippet
    });

    let data = client.post("/query_snippets", body).await?;
    Ok(format_tool_result(&data))
}

/// Get a query snippet by ID.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/query_snippets/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Update an existing query snippet.
pub async fn update(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let mut body = serde_json::json!({});

    if let Some(trigger) = optional_string(args, "trigger") {
        body["trigger"] = serde_json::json!(trigger);
    }
    if let Some(description) = optional_string(args, "description") {
        body["description"] = serde_json::json!(description);
    }
    if let Some(snippet) = optional_string(args, "snippet") {
        body["snippet"] = serde_json::json!(snippet);
    }

    let data = client.put(&format!("/query_snippets/{id}"), body).await?;
    Ok(format_tool_result(&data))
}

/// Delete a query snippet by ID.
pub async fn delete(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/query_snippets/{id}")).await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_query_snippets_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_query_snippets")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn create_query_snippet_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "create_query_snippet")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("trigger")));
        assert!(required.contains(&json!("description")));
        assert!(required.contains(&json!("snippet")));
    }

    #[test]
    fn get_query_snippet_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "get_query_snippet")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn update_query_snippet_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "update_query_snippet")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn delete_query_snippet_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "delete_query_snippet")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 5);
    }
}
