mod alerts;
mod common;
mod dashboards;
mod data_sources;
mod destinations;
mod favorites;
mod queries;
mod query_results;
mod snippets;
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
    defs.extend(alerts::definitions());
    defs.extend(snippets::definitions());
    defs.extend(favorites::definitions());
    defs.extend(destinations::definitions());
    defs
}

/// Dispatch a tool call by name to the appropriate handler.
pub async fn call_tool(name: &str, args: &Value, client: &RedashClient) -> Result<Value> {
    match name {
        "list_data_sources" => data_sources::list(client).await,
        "get_data_source" => data_sources::get(client, args).await,
        "get_data_source_schema" => data_sources::get_schema(client, args).await,
        "test_data_source" => data_sources::test(client, args).await,
        "list_queries" => queries::list(client, args).await,
        "get_query" => queries::get(client, args).await,
        "search_queries" => queries::search(client, args).await,
        "create_query" => queries::create(client, args).await,
        "update_query" => queries::update(client, args).await,
        "archive_query" => queries::archive(client, args).await,
        "refresh_query" => queries::refresh(client, args).await,
        "fork_query" => queries::fork(client, args).await,
        "list_query_tags" => queries::list_tags(client).await,
        "list_my_queries" => queries::list_my(client, args).await,
        "list_recent_queries" => queries::list_recent(client, args).await,
        "list_archived_queries" => queries::list_archived(client, args).await,
        "get_query_result" => query_results::get(client, args).await,
        "execute_query" => query_results::execute(client, args).await,
        "list_dashboards" => dashboards::list(client, args).await,
        "get_dashboard" => dashboards::get(client, args).await,
        "create_dashboard" => dashboards::create(client, args).await,
        "update_dashboard" => dashboards::update(client, args).await,
        "archive_dashboard" => dashboards::archive(client, args).await,
        "list_dashboard_tags" => dashboards::list_tags(client).await,
        "share_dashboard" => dashboards::share(client, args).await,
        "unshare_dashboard" => dashboards::unshare(client, args).await,
        "list_users" => users::list(client, args).await,
        "get_user" => users::get(client, args).await,
        "create_visualization" => visualizations::create(client, args).await,
        "update_visualization" => visualizations::update(client, args).await,
        "delete_visualization" => visualizations::delete(client, args).await,
        "add_widget" => widgets::add(client, args).await,
        "remove_widget" => widgets::remove(client, args).await,
        "list_alerts" => alerts::list(client).await,
        "get_alert" => alerts::get(client, args).await,
        "create_alert" => alerts::create(client, args).await,
        "delete_alert" => alerts::delete(client, args).await,
        "list_query_snippets" => snippets::list(client).await,
        "create_query_snippet" => snippets::create(client, args).await,
        "favorite_query" => favorites::favorite_query(client, args).await,
        "unfavorite_query" => favorites::unfavorite_query(client, args).await,
        "favorite_dashboard" => favorites::favorite_dashboard(client, args).await,
        "unfavorite_dashboard" => favorites::unfavorite_dashboard(client, args).await,
        "list_destinations" => destinations::list(client).await,
        "list_alert_subscriptions" => alerts::list_subscriptions(client, args).await,
        "create_alert_subscription" => alerts::create_subscription(client, args).await,
        _ => Err(Error::Tool(format!("unknown tool: {name}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definitions_count() {
        let defs = tool_definitions();
        assert_eq!(defs.len(), 46);
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

    #[test]
    fn all_definitions_have_annotations() {
        for def in tool_definitions() {
            let name = def["name"].as_str().unwrap();
            let ann = def.get("annotations");
            assert!(ann.is_some(), "tool {name} missing annotations");
            let ann = ann.unwrap();
            assert!(
                ann.get("readOnlyHint").is_some(),
                "tool {name} missing readOnlyHint"
            );
            assert!(
                ann.get("destructiveHint").is_some(),
                "tool {name} missing destructiveHint"
            );
            assert!(
                ann.get("idempotentHint").is_some(),
                "tool {name} missing idempotentHint"
            );
            assert!(
                ann.get("openWorldHint").is_some(),
                "tool {name} missing openWorldHint"
            );
        }
    }

    #[test]
    fn read_only_tools_not_destructive() {
        for def in tool_definitions() {
            let name = def["name"].as_str().unwrap();
            let ann = &def["annotations"];
            if ann["readOnlyHint"].as_bool() == Some(true) {
                assert_eq!(
                    ann["destructiveHint"].as_bool(),
                    Some(false),
                    "tool {name} is readOnly but also destructive"
                );
            }
        }
    }
}
