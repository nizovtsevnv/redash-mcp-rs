use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{format_tool_result, optional_u64, required_string};
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
            }
        }),
        serde_json::json!({
            "name": "list_dashboard_tags",
            "description": "List all tags used on dashboards",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
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

/// List all dashboard tags.
pub async fn list_tags(client: &RedashClient) -> Result<Value> {
    let data = client.get("/dashboards/tags").await?;
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
    fn list_dashboard_tags_definition_no_required() {
        let defs = definitions();
        let tags_def = defs
            .iter()
            .find(|d| d["name"] == "list_dashboard_tags")
            .unwrap();
        let required = tags_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }
}
