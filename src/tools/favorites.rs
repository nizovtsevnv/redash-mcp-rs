use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_u64, required_u64};
use serde_json::Value;

/// Tool definitions for favorite tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "favorite_query",
            "description": "Mark a query as favorite",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID"
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
            "name": "unfavorite_query",
            "description": "Remove a query from favorites",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID"
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
            "name": "favorite_dashboard",
            "description": "Mark a dashboard as favorite",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Dashboard ID"
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
            "name": "unfavorite_dashboard",
            "description": "Remove a dashboard from favorites",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Dashboard ID"
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
            "name": "list_favorite_queries",
            "description": "List queries marked as favorite",
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
            "name": "list_favorite_dashboards",
            "description": "List dashboards marked as favorite",
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
    ]
}

/// Mark a query as favorite.
pub async fn favorite_query(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/queries/{id}/favorite"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

/// Remove a query from favorites.
pub async fn unfavorite_query(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/queries/{id}/favorite")).await?;
    Ok(format_tool_result(&data))
}

/// Mark a dashboard as favorite.
pub async fn favorite_dashboard(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/dashboards/{id}/favorite"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

/// List queries marked as favorite.
pub async fn list_favorite_queries(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!(
            "/queries/favorites?page={page}&page_size={page_size}"
        ))
        .await?;
    Ok(format_tool_result(&data))
}

/// List dashboards marked as favorite.
pub async fn list_favorite_dashboards(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!(
            "/dashboards/favorites?page={page}&page_size={page_size}"
        ))
        .await?;
    Ok(format_tool_result(&data))
}

/// Remove a dashboard from favorites.
pub async fn unfavorite_dashboard(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/dashboards/{id}/favorite")).await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn favorite_query_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "favorite_query").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn unfavorite_query_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "unfavorite_query")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn favorite_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "favorite_dashboard")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn unfavorite_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "unfavorite_dashboard")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 6);
    }

    #[test]
    fn list_favorite_queries_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_favorite_queries")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn list_favorite_dashboards_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_favorite_dashboards")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn all_definitions_have_unique_names() {
        let defs = definitions();
        let names: Vec<&str> = defs.iter().map(|d| d["name"].as_str().unwrap()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len());
    }
}
