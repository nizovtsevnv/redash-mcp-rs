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
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
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
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
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
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "test_data_source",
            "description": "Test connectivity to a data source",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Data source ID"
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
            "name": "list_data_source_types",
            "description": "List all available data source types",
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

/// List all available data source types.
pub async fn list_types(client: &RedashClient) -> Result<Value> {
    let data = client.get("/data_sources/types").await?;
    Ok(format_tool_result(&data))
}

/// Test connectivity to a data source.
pub async fn test(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/data_sources/{id}/test"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn list_data_sources_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_data_sources")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn get_data_source_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "get_data_source")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn get_data_source_schema_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "get_data_source_schema")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn test_data_source_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "test_data_source")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 5);
    }

    #[test]
    fn list_data_source_types_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_data_source_types")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn all_definitions_have_input_schema() {
        for def in definitions() {
            assert!(
                def.get("inputSchema").is_some(),
                "definition missing inputSchema: {def}"
            );
            assert!(
                def["inputSchema"].get("type").is_some(),
                "inputSchema missing type: {def}"
            );
        }
    }

    #[test]
    fn all_definitions_have_unique_names() {
        let defs = definitions();
        let names: Vec<&str> = defs.iter().map(|d| d["name"].as_str().unwrap()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "duplicate tool names found");
    }
}
