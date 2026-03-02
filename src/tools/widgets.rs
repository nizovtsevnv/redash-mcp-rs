use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_json, optional_string, required_u64};
use serde_json::Value;

/// Tool definitions for widget tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "add_widget",
            "description": "Add a widget to a dashboard",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "dashboard_id": {
                        "type": "integer",
                        "description": "Dashboard ID"
                    },
                    "visualization_id": {
                        "type": "integer",
                        "description": "Visualization ID to display in the widget"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text content for a text widget"
                    },
                    "options": {
                        "type": "object",
                        "description": "Widget layout and display options"
                    }
                },
                "required": ["dashboard_id"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "remove_widget",
            "description": "Remove a widget from a dashboard",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Widget ID"
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
        serde_json::json!({
            "name": "update_widget",
            "description": "Update an existing widget",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Widget ID"
                    },
                    "options": {
                        "type": "object",
                        "description": "Widget layout and display options"
                    },
                    "text": {
                        "type": "string",
                        "description": "Text content for a text widget"
                    },
                    "visualization_id": {
                        "type": "integer",
                        "description": "Visualization ID to display"
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
    ]
}

/// Add a widget to a dashboard.
pub async fn add(client: &RedashClient, args: &Value) -> Result<Value> {
    let dashboard_id = required_u64(args, "dashboard_id")?;

    let mut body = serde_json::json!({
        "dashboard_id": dashboard_id
    });

    if let Some(visualization_id) = args.get("visualization_id").and_then(|v| v.as_u64()) {
        body["visualization_id"] = serde_json::json!(visualization_id);
    }
    if let Some(text) = optional_string(args, "text") {
        body["text"] = serde_json::json!(text);
    }
    if let Some(options) = optional_json(args, "options") {
        body["options"] = options;
    }

    let data = client.post("/widgets", body).await?;
    Ok(format_tool_result(&data))
}

/// Update an existing widget.
pub async fn update(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let mut body = serde_json::json!({});

    if let Some(options) = optional_json(args, "options") {
        body["options"] = options;
    }
    if let Some(text) = optional_string(args, "text") {
        body["text"] = serde_json::json!(text);
    }
    if let Some(visualization_id) = args.get("visualization_id").and_then(|v| v.as_u64()) {
        body["visualization_id"] = serde_json::json!(visualization_id);
    }

    let data = client.put(&format!("/widgets/{id}"), body).await?;
    Ok(format_tool_result(&data))
}

/// Remove a widget from a dashboard.
pub async fn remove(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/widgets/{id}")).await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn add_widget_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "add_widget").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("dashboard_id")));
        assert!(!required.contains(&json!("visualization_id")));
    }

    #[test]
    fn remove_widget_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "remove_widget").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn update_widget_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "update_widget").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 3);
    }
}
