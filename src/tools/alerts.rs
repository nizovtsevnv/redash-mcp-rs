use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, required_json, required_string, required_u64};
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
    fn definitions_count() {
        assert_eq!(definitions().len(), 4);
    }
}
