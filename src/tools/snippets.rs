use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, required_string};
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
    fn definitions_count() {
        assert_eq!(definitions().len(), 2);
    }
}
