mod common;
mod dashboards;
mod data_sources;
mod queries;
mod query_results;
mod users;
mod visualizations;
mod widgets;

pub use common::format_tool_error;

use crate::error::{Error, Result};
use crate::redash::RedashClient;
use serde_json::Value;

/// Return definitions for all registered tools.
pub fn tool_definitions() -> Vec<Value> {
    let mut defs = Vec::new();
    defs.extend(data_sources::definitions());
    defs.extend(queries::definitions());
    defs.extend(query_results::definitions());
    defs.extend(dashboards::definitions());
    defs.extend(users::definitions());
    defs.extend(visualizations::definitions());
    defs.extend(widgets::definitions());
    defs
}

/// Dispatch a tool call by name to the appropriate handler.
pub async fn call_tool(name: &str, args: &Value, client: &RedashClient) -> Result<Value> {
    match name {
        "list_data_sources" => data_sources::list(client).await,
        "get_data_source" => data_sources::get(client, args).await,
        "get_data_source_schema" => data_sources::get_schema(client, args).await,
        "list_queries" => queries::list(client, args).await,
        "get_query" => queries::get(client, args).await,
        "search_queries" => queries::search(client, args).await,
        "create_query" => queries::create(client, args).await,
        "update_query" => queries::update(client, args).await,
        "archive_query" => queries::archive(client, args).await,
        "list_query_tags" => queries::list_tags(client).await,
        "get_query_result" => query_results::get(client, args).await,
        "execute_query" => query_results::execute(client, args).await,
        "list_dashboards" => dashboards::list(client, args).await,
        "get_dashboard" => dashboards::get(client, args).await,
        "create_dashboard" => dashboards::create(client, args).await,
        "list_dashboard_tags" => dashboards::list_tags(client).await,
        "list_users" => users::list(client, args).await,
        "get_user" => users::get(client, args).await,
        "create_visualization" => visualizations::create(client, args).await,
        "update_visualization" => visualizations::update(client, args).await,
        "delete_visualization" => visualizations::delete(client, args).await,
        "add_widget" => widgets::add(client, args).await,
        "remove_widget" => widgets::remove(client, args).await,
        _ => Err(Error::Tool(format!("unknown tool: {name}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definitions_count() {
        let defs = tool_definitions();
        assert_eq!(defs.len(), 23);
    }

    #[test]
    fn all_definitions_have_required_fields() {
        for def in tool_definitions() {
            assert!(
                def.get("name").and_then(|v| v.as_str()).is_some(),
                "tool definition missing name: {def}"
            );
            assert!(
                def.get("description").and_then(|v| v.as_str()).is_some(),
                "tool definition missing description: {def}"
            );
            assert!(
                def.get("inputSchema").is_some(),
                "tool definition missing inputSchema: {def}"
            );
        }
    }

    #[test]
    fn all_definitions_have_unique_names() {
        let defs = tool_definitions();
        let names: Vec<&str> = defs.iter().map(|d| d["name"].as_str().unwrap()).collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len(), "duplicate tool names found");
    }
}
