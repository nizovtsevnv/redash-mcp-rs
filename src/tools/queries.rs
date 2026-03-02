use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::{
    format_tool_result, optional_json, optional_string, optional_u64, required_string,
    required_u64, truncate_query_result,
};
use serde_json::Value;

/// Tool definitions for query tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "list_queries",
            "description": "List saved queries with optional pagination",
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
            "name": "get_query",
            "description": "Get details of a specific query by ID, including the SQL",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "search_queries",
            "description": "Search queries by name or description",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "q": {
                        "type": "string",
                        "description": "Search term"
                    },
                    "page": {
                        "type": "integer",
                        "description": "Page number (default: 1)"
                    },
                    "page_size": {
                        "type": "integer",
                        "description": "Results per page (default: 25)"
                    }
                },
                "required": ["q"]
            }
        }),
        serde_json::json!({
            "name": "create_query",
            "description": "Create a new saved query",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "description": "Query name"
                    },
                    "query": {
                        "type": "string",
                        "description": "SQL query text"
                    },
                    "data_source_id": {
                        "type": "integer",
                        "description": "Data source ID to run the query against"
                    },
                    "description": {
                        "type": "string",
                        "description": "Query description"
                    },
                    "options": {
                        "type": "object",
                        "description": "Additional query options"
                    }
                },
                "required": ["name", "query", "data_source_id"]
            }
        }),
        serde_json::json!({
            "name": "update_query",
            "description": "Update an existing query's name, description, or SQL",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID"
                    },
                    "name": {
                        "type": "string",
                        "description": "New query name"
                    },
                    "query": {
                        "type": "string",
                        "description": "New SQL query text"
                    },
                    "description": {
                        "type": "string",
                        "description": "New query description"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "archive_query",
            "description": "Archive a query by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "refresh_query",
            "description": "Refresh a saved query by executing it and returning new results. Polls for completion if async.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID"
                    },
                    "max_rows": {
                        "type": "integer",
                        "description": "Maximum number of rows to return (default: 100)"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "fork_query",
            "description": "Fork (duplicate) a query by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "Query ID to fork"
                    }
                },
                "required": ["id"]
            }
        }),
        serde_json::json!({
            "name": "list_query_tags",
            "description": "List all tags used on queries",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "required": []
            }
        }),
    ]
}

/// List saved queries with optional pagination.
pub async fn list(client: &RedashClient, args: &Value) -> Result<Value> {
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!("/queries?page={page}&page_size={page_size}"))
        .await?;
    Ok(format_tool_result(&data))
}

/// Get a specific query by ID.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.get(&format!("/queries/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Search queries by name or description.
pub async fn search(client: &RedashClient, args: &Value) -> Result<Value> {
    let q = required_string(args, "q")?;
    let page = optional_u64(args, "page", 1);
    let page_size = optional_u64(args, "page_size", 25);
    let data = client
        .get(&format!(
            "/queries/search?q={}&page={page}&page_size={page_size}",
            urlencoded(&q)
        ))
        .await?;
    Ok(format_tool_result(&data))
}

/// Create a new saved query.
pub async fn create(client: &RedashClient, args: &Value) -> Result<Value> {
    let name = required_string(args, "name")?;
    let query = required_string(args, "query")?;
    let data_source_id = required_u64(args, "data_source_id")?;

    let mut body = serde_json::json!({
        "name": name,
        "query": query,
        "data_source_id": data_source_id
    });

    if let Some(description) = optional_string(args, "description") {
        body["description"] = serde_json::json!(description);
    }
    if let Some(options) = optional_json(args, "options") {
        body["options"] = options;
    }

    let data = client.post("/queries", body).await?;
    Ok(format_tool_result(&data))
}

/// Update an existing query.
pub async fn update(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;

    let mut body = serde_json::json!({});
    if let Some(name) = optional_string(args, "name") {
        body["name"] = serde_json::json!(name);
    }
    if let Some(query) = optional_string(args, "query") {
        body["query"] = serde_json::json!(query);
    }
    if let Some(description) = optional_string(args, "description") {
        body["description"] = serde_json::json!(description);
    }

    let data = client.post(&format!("/queries/{id}"), body).await?;
    Ok(format_tool_result(&data))
}

/// Archive a query by ID.
pub async fn archive(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client.delete(&format!("/queries/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Refresh a saved query by executing it and returning new results.
pub async fn refresh(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let max_rows = optional_u64(args, "max_rows", 100);

    let data = client
        .post(&format!("/queries/{id}/refresh"), serde_json::json!({}))
        .await?;

    let result = if data.get("job").is_some() {
        super::query_results::poll_job(client, &data).await?
    } else {
        data
    };

    let truncated = truncate_query_result(&result, max_rows);
    Ok(format_tool_result(&truncated))
}

/// Fork (duplicate) a query by ID.
pub async fn fork(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let data = client
        .post(&format!("/queries/{id}/fork"), serde_json::json!({}))
        .await?;
    Ok(format_tool_result(&data))
}

/// List all query tags.
pub async fn list_tags(client: &RedashClient) -> Result<Value> {
    let data = client.get("/queries/tags").await?;
    Ok(format_tool_result(&data))
}

/// Percent-encode a query string value.
fn urlencoded(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn create_query_definition_required_fields() {
        let defs = definitions();
        let create_def = defs.iter().find(|d| d["name"] == "create_query").unwrap();
        let required = create_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("name")));
        assert!(required.contains(&json!("query")));
        assert!(required.contains(&json!("data_source_id")));
    }

    #[test]
    fn update_query_definition_required_fields() {
        let defs = definitions();
        let update_def = defs.iter().find(|d| d["name"] == "update_query").unwrap();
        let required = update_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
        assert!(!required.contains(&json!("name")));
    }

    #[test]
    fn archive_query_definition_required_fields() {
        let defs = definitions();
        let archive_def = defs.iter().find(|d| d["name"] == "archive_query").unwrap();
        let required = archive_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn refresh_query_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "refresh_query").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
        assert!(!required.contains(&json!("max_rows")));
    }

    #[test]
    fn fork_query_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "fork_query").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn list_query_tags_definition_no_required() {
        let defs = definitions();
        let tags_def = defs
            .iter()
            .find(|d| d["name"] == "list_query_tags")
            .unwrap();
        let required = tags_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }
}
