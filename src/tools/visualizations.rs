use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{
    format_tool_result, optional_json, optional_string, required_string, required_u64,
};
use serde_json::Value;

/// Tool definitions for visualization tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "create_visualization",
            "description": "Create a new visualization for a query",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query_id": {
                        "type": "integer",
                        "description": "Query ID to attach the visualization to"
                    },
                    "type": {
                        "type": "string",
                        "description": "Visualization type (e.g. TABLE, CHART, COUNTER, MAP)"
                    },
                    "name": {
                        "type": "string",
                        "description": "Visualization name"
                    },
                    "options": {
                        "type": "object",
                        "description": "Visualization-specific options"
                    }
                },
                "required": ["query_id", "type", "name"]
            }
        }),
        serde_json::json!({
            "name": "update_visualization",
            "description": "Update an existing visualization",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Visualization ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "New visualization name"
                    },
                    "options": {
                        "type": "object",
                        "description": "New visualization options"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "delete_visualization",
            "description": "Delete a visualization by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Visualization ID"
                    }
                },
                "required": ["id"]
            }
        }),
    ]
}

/// Create a new visualization for a query.
pub async fn create(client: &RedashClient, args: &Value) -> Result<Value> {
    let query_id = required_u64(args, "query_id")?;
    let vis_type = required_string(args, "type")?;
    let name = required_string(args, "name")?;

    let mut body = serde_json::json!({
        "query_id": query_id,
        "type": vis_type,
        "name": name
    });

    if let Some(options) = optional_json(args, "options") {
        body["options"] = options;
    }

    let data = client.post("/visualizations", body).await?;
    Ok(format_tool_result(&data))
}

/// Update an existing visualization.
pub async fn update(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;

    let mut body = serde_json::json!({});
    if let Some(name) = optional_string(args, "name") {
        body["name"] = serde_json::json!(name);
    }
    if let Some(options) = optional_json(args, "options") {
        body["options"] = options;
    }

    let data = client.post(&format!("/visualizations/{id}"), body).await?;
    Ok(format_tool_result(&data))
}

/// Delete a visualization by ID.
pub async fn delete(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/visualizations/{id}")).await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_visualization_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "create_visualization")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("query_id")));
        assert!(required.contains(&json!("type")));
        assert!(required.contains(&json!("name")));
    }

    #[test]
    fn update_visualization_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "update_visualization")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
        assert!(!required.contains(&json!("name")));
    }

    #[test]
    fn delete_visualization_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "delete_visualization")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 3);
    }
}
