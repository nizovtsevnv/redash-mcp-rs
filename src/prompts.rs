use crate::error::{Error, Result};
use serde_json::Value;

/// Return the list of available prompts.
pub fn prompt_list() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "explore_data",
            "description": "Explore a data source: fetch schema, write an exploratory query, execute it, and analyze results",
            "arguments": [
                {
                    "name": "data_source_id",
                    "description": "Data source ID to explore",
                    "required": true
                }
            ]
        }),
        serde_json::json!({
            "name": "build_dashboard",
            "description": "Build a dashboard: list queries, create a dashboard, add visualizations and widgets",
            "arguments": [
                {
                    "name": "dashboard_name",
                    "description": "Name for the new dashboard",
                    "required": true
                },
                {
                    "name": "query_ids",
                    "description": "Comma-separated query IDs to include (optional — will list queries if omitted)",
                    "required": false
                }
            ]
        }),
        serde_json::json!({
            "name": "setup_alert",
            "description": "Set up an alert: inspect a query, check its results, configure a threshold, and create an alert",
            "arguments": [
                {
                    "name": "query_id",
                    "description": "Query ID to set up an alert on",
                    "required": true
                }
            ]
        }),
    ]
}

/// Get a specific prompt by name, rendering its messages with the given arguments.
pub fn get_prompt(name: &str, args: &Value) -> Result<Value> {
    match name {
        "explore_data" => get_explore_data(args),
        "build_dashboard" => get_build_dashboard(args),
        "setup_alert" => get_setup_alert(args),
        _ => Err(Error::Tool(format!("unknown prompt: {name}"))),
    }
}

fn get_explore_data(args: &Value) -> Result<Value> {
    let data_source_id = args
        .get("data_source_id")
        .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
        .ok_or_else(|| Error::Tool("missing required argument: data_source_id".into()))?;

    let id_str = if data_source_id.is_empty() {
        args["data_source_id"].to_string()
    } else {
        data_source_id.to_string()
    };

    Ok(serde_json::json!({
        "description": "Explore a data source by fetching its schema, writing a query, and analyzing results",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "I want to explore data source {}. Please:\n\
                         1. Use get_data_source_schema to fetch the schema for data source {}\n\
                         2. Examine the tables and columns available\n\
                         3. Write an exploratory SQL query to understand the data (e.g. row counts, sample data)\n\
                         4. Use execute_query to run it against data source {}\n\
                         5. Analyze the results and suggest interesting queries to run next",
                        id_str, id_str, id_str
                    )
                }
            }
        ]
    }))
}

fn get_build_dashboard(args: &Value) -> Result<Value> {
    let dashboard_name = args
        .get("dashboard_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Tool("missing required argument: dashboard_name".into()))?;

    let query_ids_hint = match args.get("query_ids").and_then(|v| v.as_str()) {
        Some(ids) => format!("Use these query IDs: {ids}"),
        None => "Use list_queries to find relevant queries".to_string(),
    };

    Ok(serde_json::json!({
        "description": "Build a dashboard with visualizations and widgets",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "I want to build a dashboard called \"{dashboard_name}\". Please:\n\
                         1. {query_ids_hint}\n\
                         2. Use create_dashboard to create a dashboard named \"{dashboard_name}\"\n\
                         3. For each query, use create_visualization to create appropriate visualizations\n\
                         4. Use add_widget to add each visualization to the dashboard\n\
                         5. Summarize the final dashboard layout"
                    )
                }
            }
        ]
    }))
}

fn get_setup_alert(args: &Value) -> Result<Value> {
    let query_id = args
        .get("query_id")
        .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|_| "")))
        .ok_or_else(|| Error::Tool("missing required argument: query_id".into()))?;

    let id_str = if query_id.is_empty() {
        args["query_id"].to_string()
    } else {
        query_id.to_string()
    };

    Ok(serde_json::json!({
        "description": "Set up an alert on a query with a threshold condition",
        "messages": [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": format!(
                        "I want to set up an alert on query {id_str}. Please:\n\
                         1. Use get_query to inspect query {id_str} and understand what it does\n\
                         2. Use get_query_result to check the latest results and available columns\n\
                         3. Suggest a meaningful threshold condition (column, operator, value)\n\
                         4. Ask me to confirm the alert configuration\n\
                         5. Use create_alert to create the alert with the confirmed options"
                    )
                }
            }
        ]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn prompt_list_count() {
        assert_eq!(prompt_list().len(), 3);
    }

    #[test]
    fn prompt_list_has_required_fields() {
        for prompt in prompt_list() {
            assert!(prompt.get("name").is_some());
            assert!(prompt.get("description").is_some());
            assert!(prompt.get("arguments").is_some());
        }
    }

    #[test]
    fn prompt_list_has_unique_names() {
        let prompts = prompt_list();
        let names: Vec<&str> = prompts
            .iter()
            .map(|p| p["name"].as_str().unwrap())
            .collect();
        let mut unique = names.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(names.len(), unique.len());
    }

    #[test]
    fn get_explore_data_prompt() {
        let args = json!({"data_source_id": "1"});
        let result = get_prompt("explore_data", &args).unwrap();
        assert!(result.get("description").is_some());
        let messages = result["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        let text = messages[0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("data source 1"));
        assert!(text.contains("get_data_source_schema"));
    }

    #[test]
    fn get_explore_data_numeric_id() {
        let args = json!({"data_source_id": 42});
        let result = get_prompt("explore_data", &args).unwrap();
        let text = result["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("42"));
    }

    #[test]
    fn get_explore_data_missing_arg() {
        let args = json!({});
        let err = get_prompt("explore_data", &args).unwrap_err();
        assert!(err.to_string().contains("data_source_id"));
    }

    #[test]
    fn get_build_dashboard_prompt() {
        let args = json!({"dashboard_name": "Sales Overview"});
        let result = get_prompt("build_dashboard", &args).unwrap();
        let text = result["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("Sales Overview"));
        assert!(text.contains("list_queries"));
    }

    #[test]
    fn get_build_dashboard_with_query_ids() {
        let args = json!({"dashboard_name": "KPI", "query_ids": "1,2,3"});
        let result = get_prompt("build_dashboard", &args).unwrap();
        let text = result["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("1,2,3"));
        assert!(!text.contains("list_queries"));
    }

    #[test]
    fn get_build_dashboard_missing_name() {
        let args = json!({});
        let err = get_prompt("build_dashboard", &args).unwrap_err();
        assert!(err.to_string().contains("dashboard_name"));
    }

    #[test]
    fn get_setup_alert_prompt() {
        let args = json!({"query_id": "5"});
        let result = get_prompt("setup_alert", &args).unwrap();
        let text = result["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("query 5"));
        assert!(text.contains("create_alert"));
    }

    #[test]
    fn get_setup_alert_numeric_id() {
        let args = json!({"query_id": 10});
        let result = get_prompt("setup_alert", &args).unwrap();
        let text = result["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(text.contains("10"));
    }

    #[test]
    fn get_setup_alert_missing_arg() {
        let args = json!({});
        let err = get_prompt("setup_alert", &args).unwrap_err();
        assert!(err.to_string().contains("query_id"));
    }

    #[test]
    fn get_unknown_prompt() {
        let args = json!({});
        let err = get_prompt("nonexistent", &args).unwrap_err();
        assert!(err.to_string().contains("unknown prompt"));
    }
}
