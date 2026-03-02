use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{
    format_tool_result, optional_json, optional_string, required_json, required_string,
    required_u64,
};
use serde_json::Value;

/// Tool definitions for alert tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_alerts",
            "description": "List all alerts",
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
            "name": "get_alert",
            "description": "Get an alert by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Alert ID"
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
            "name": "create_alert",
            "description": "Create a new alert on a query",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query_id": {
                        "type": "integer",
                        "description": "Query ID to monitor"
                    },
                    "name": {
                        "type": "string",
                        "description": "Alert name"
                    },
                    "options": {
                        "type": "object",
                        "description": "Alert options: {\"column\": \"...\", \"op\": \"greater than\", \"value\": 100}",
                        "properties": {
                            "column": {
                                "type": "string",
                                "description": "Column to monitor"
                            },
                            "op": {
                                "type": "string",
                                "description": "Comparison operator (e.g. greater than, less than, equals)"
                            },
                            "value": {
                                "type": "number",
                                "description": "Threshold value"
                            }
                        },
                        "required": ["column", "op", "value"]
                    }
                },
                "required": ["query_id", "name", "options"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "delete_alert",
            "description": "Delete an alert by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Alert ID"
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
            "name": "list_alert_subscriptions",
            "description": "List subscriptions for an alert",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "alert_id": {
                        "type": "integer",
                        "description": "Alert ID"
                    }
                },
                "required": ["alert_id"]
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "create_alert_subscription",
            "description": "Subscribe a destination to an alert",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "alert_id": {
                        "type": "integer",
                        "description": "Alert ID"
                    },
                    "destination_id": {
                        "type": "integer",
                        "description": "Destination ID"
                    }
                },
                "required": ["alert_id", "destination_id"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "update_alert",
            "description": "Update an existing alert",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Alert ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "New alert name"
                    },
                    "options": {
                        "type": "object",
                        "description": "New alert options: {\"column\": \"...\", \"op\": \"greater than\", \"value\": 100}"
                    },
                    "query_id": {
                        "type": "integer",
                        "description": "New query ID to monitor"
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
            "name": "mute_alert",
            "description": "Mute an alert to temporarily stop notifications",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Alert ID"
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

/// List all alerts.
pub async fn list(client: &RedashClient) -> Result<Value> {
    let data = client.get("/alerts").await?;
    Ok(format_tool_result(&data))
}

/// Get an alert by ID.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/alerts/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Create a new alert.
pub async fn create(client: &RedashClient, args: &Value) -> Result<Value> {
    let query_id = required_u64(args, "query_id")?;
    let name = required_string(args, "name")?;
    let options = required_json(args, "options")?;

    let body = serde_json::json!({
        "query_id": query_id,
        "name": name,
        "options": options
    });

    let data = client.post("/alerts", body).await?;
    Ok(format_tool_result(&data))
}

/// Delete an alert by ID.
pub async fn delete(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/alerts/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// List subscriptions for an alert.
pub async fn list_subscriptions(client: &RedashClient, args: &Value) -> Result<Value> {
    let alert_id = required_u64(args, "alert_id")?;
    let data = client
        .get(&format!("/alerts/{alert_id}/subscriptions"))
        .await?;
    Ok(format_tool_result(&data))
}

/// Update an existing alert.
pub async fn update(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let mut body = serde_json::json!({});

    if let Some(name) = optional_string(args, "name") {
        body["name"] = serde_json::json!(name);
    }
    if let Some(options) = optional_json(args, "options") {
        body["options"] = options;
    }
    if let Some(query_id) = args.get("query_id").and_then(|v| v.as_u64()) {
        body["query_id"] = serde_json::json!(query_id);
    }

    let data = client.put(&format!("/alerts/{id}"), body).await?;
    Ok(format_tool_result(&data))
}

/// Mute an alert to temporarily stop notifications.
pub async fn mute(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/alerts/{id}/mute"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

/// Subscribe a destination to an alert.
pub async fn create_subscription(client: &RedashClient, args: &Value) -> Result<Value> {
    let alert_id = required_u64(args, "alert_id")?;
    let destination_id = required_u64(args, "destination_id")?;
    let body = serde_json::json!({ "destination_id": destination_id });
    let data = client
        .post(&format!("/alerts/{alert_id}/subscriptions"), body)
        .await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_alerts_definition_no_required() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "list_alerts").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn get_alert_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "get_alert").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn create_alert_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "create_alert").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("query_id")));
        assert!(required.contains(&json!("name")));
        assert!(required.contains(&json!("options")));
    }

    #[test]
    fn delete_alert_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "delete_alert").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn list_alert_subscriptions_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_alert_subscriptions")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("alert_id")));
    }

    #[test]
    fn create_alert_subscription_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "create_alert_subscription")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("alert_id")));
        assert!(required.contains(&json!("destination_id")));
    }

    #[test]
    fn update_alert_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "update_alert").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn mute_alert_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "mute_alert").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 8);
    }
}
