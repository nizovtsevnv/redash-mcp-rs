use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{
    format_tool_result, optional_array, optional_string, optional_u64, required_string,
    required_u64,
};
use serde_json::Value;

/// Tool definitions for dashboard tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_dashboards",
            "description": "List dashboards with optional pagination",
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
            "name": "get_dashboard",
            "description": "Get a dashboard with its widgets by slug",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "slug": {
                        "type": "string",
                        "description": "Dashboard slug"
                    }
                },
                "required": ["slug"]
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "create_dashboard",
            "description": "Create a new dashboard",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Dashboard name"
                    }
                },
                "required": ["name"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "update_dashboard",
            "description": "Update an existing dashboard",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Dashboard ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "New dashboard name"
                    },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Dashboard tags"
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
            "name": "archive_dashboard",
            "description": "Archive a dashboard by ID",
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
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "list_dashboard_tags",
            "description": "List all tags used on dashboards",
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
            "name": "share_dashboard",
            "description": "Enable public sharing for a dashboard and get a public URL",
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
            "name": "unshare_dashboard",
            "description": "Disable public sharing for a dashboard",
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
                "destructiveHint": true,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "list_my_dashboards",
            "description": "List dashboards owned by the current user",
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
            "name": "fork_dashboard",
            "description": "Fork (copy) an existing dashboard",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Dashboard ID to fork"
                    }
                },
                "required": ["id"]
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

/// List dashboards with optional pagination.
pub async fn list(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!("/dashboards?page={page}&page_size={page_size}"))
        .await?;
    Ok(format_tool_result(&data))
}

/// Get a dashboard by slug.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let slug = required_string(args, "slug")?;
    let data = client.get(&format!("/dashboards/{slug}")).await?;
    Ok(format_tool_result(&data))
}

/// Create a new dashboard.
pub async fn create(client: &RedashClient, args: &Value) -> Result<Value> {
    let name = required_string(args, "name")?;
    let body = serde_json::json!({ "name": name });
    let data = client.post("/dashboards", body).await?;
    Ok(format_tool_result(&data))
}

/// Update an existing dashboard.
pub async fn update(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;

    let mut body = serde_json::json!({});
    if let Some(name) = optional_string(args, "name") {
        body["name"] = serde_json::json!(name);
    }
    if let Some(tags) = optional_array(args, "tags") {
        body["tags"] = serde_json::json!(tags);
    }

    let data = client.post(&format!("/dashboards/{id}"), body).await?;
    Ok(format_tool_result(&data))
}

/// Archive a dashboard by ID.
pub async fn archive(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/dashboards/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// List dashboards owned by the current user.
pub async fn list_my(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!("/dashboards/my?page={page}&page_size={page_size}"))
        .await?;
    Ok(format_tool_result(&data))
}

/// List all dashboard tags.
pub async fn list_tags(client: &RedashClient) -> Result<Value> {
    let data = client.get("/dashboards/tags").await?;
    Ok(format_tool_result(&data))
}

/// Enable public sharing for a dashboard.
pub async fn share(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/dashboards/{id}/share"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

/// Fork (copy) an existing dashboard.
pub async fn fork(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/dashboards/{id}/fork"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

/// Disable public sharing for a dashboard.
pub async fn unshare(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/dashboards/{id}/share")).await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_dashboard_definition_required_fields() {
        let defs = definitions();
        let create_def = defs
            .iter()
            .find(|d| d["name"] == "create_dashboard")
            .unwrap();
        let required = create_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("name")));
    }

    #[test]
    fn update_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "update_dashboard")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
        assert!(!required.contains(&json!("name")));
    }

    #[test]
    fn archive_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "archive_dashboard")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn list_dashboard_tags_definition_no_required() {
        let defs = definitions();
        let tags_def = defs
            .iter()
            .find(|d| d["name"] == "list_dashboard_tags")
            .unwrap();
        let required = tags_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn share_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "share_dashboard")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn unshare_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "unshare_dashboard")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn fork_dashboard_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "fork_dashboard").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 10);
    }

    #[test]
    fn list_my_dashboards_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_my_dashboards")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }
}
