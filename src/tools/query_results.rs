use std::time::{Duration, Instant};

use crate::error::{Error, Result};
use crate::redash::RedashClient;
use crate::tools::common::{
    format_tool_result, optional_json, optional_u64, required_string, required_u64,
    truncate_query_result,
};
use serde_json::Value;

/// Polling constants for execute_query job status.
pub(super) const POLL_TIMEOUT: Duration = Duration::from_secs(120);
pub(super) const POLL_INITIAL: Duration = Duration::from_millis(500);
pub(super) const POLL_MAX: Duration = Duration::from_secs(5);
pub(super) const POLL_BACKOFF: f64 = 1.5;

/// Tool definitions for query result tools.
pub fn definitions() -> Vec<Value> {
    vec![
        serde_json::json!({
            "name": "get_query_result",
            "description": "Get the latest cached result of a query. Results are truncated to max_rows (default 100) with metadata.",
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
            },
            "annotations": {
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            }
        }),
        serde_json::json!({
            "name": "execute_query",
            "description": "Execute a query against a data source and return results. Polls for completion if the query is async. Results are truncated to max_rows (default 100) with metadata.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "data_source_id": {
                        "type": "integer",
                        "description": "Data source ID to execute against"
                    },
                    "query": {
                        "type": "string",
                        "description": "SQL query to execute"
                    },
                    "max_age": {
                        "type": "integer",
                        "description": "Maximum age in seconds for cached results (0 = force fresh)"
                    },
                    "max_rows": {
                        "type": "integer",
                        "description": "Maximum number of rows to return (default: 100)"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Query parameters as key-value pairs"
                    }
                },
                "required": ["data_source_id", "query"]
            },
            "annotations": {
                "readOnlyHint": false,
                "destructiveHint": false,
                "idempotentHint": false,
                "openWorldHint": true
            }
        }),
        serde_json::json!({
            "name": "get_job_status",
            "description": "Get the status of an async job by ID",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Job ID"
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
    ]
}

/// Get the latest cached result of a query.
pub async fn get(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_u64(args, "id")?;
    let max_rows = optional_u64(args, "max_rows", 100);
    let data = client.get(&format!("/queries/{id}/results.json")).await?;
    let truncated = truncate_query_result(&data, max_rows);
    Ok(format_tool_result(&truncated))
}

/// Get the status of an async job by ID.
pub async fn get_job(client: &RedashClient, args: &Value) -> Result<Value> {
    let id = required_string(args, "id")?;
    let data = client.get(&format!("/jobs/{id}")).await?;
    Ok(format_tool_result(&data))
}

/// Execute a query and return results, polling for async jobs.
pub async fn execute(client: &RedashClient, args: &Value) -> Result<Value> {
    let data_source_id = required_u64(args, "data_source_id")?;
    let query = required_string(args, "query")?;
    let max_rows = optional_u64(args, "max_rows", 100);

    let mut body = serde_json::json!({
        "data_source_id": data_source_id,
        "query": query
    });

    if let Some(max_age) = args.get("max_age").and_then(|v| v.as_u64()) {
        body["max_age"] = serde_json::json!(max_age);
    }
    if let Some(parameters) = optional_json(args, "parameters") {
        body["parameters"] = parameters;
    }

    let data = client.post("/query_results", body).await?;

    let result = if data.get("job").is_some() {
        poll_job(client, &data).await?
    } else {
        data
    };

    let truncated = truncate_query_result(&result, max_rows);
    Ok(format_tool_result(&truncated))
}

/// Poll a Redash job until completion, then fetch the result.
pub(super) async fn poll_job(client: &RedashClient, initial: &Value) -> Result<Value> {
    let job_id = initial["job"]["id"]
        .as_str()
        .ok_or_else(|| Error::Tool("missing job ID in response".to_string()))?;

    let start = Instant::now();
    let mut interval = POLL_INITIAL;

    loop {
        tokio::time::sleep(interval).await;

        if start.elapsed() > POLL_TIMEOUT {
            return Err(Error::Tool(format!(
                "query execution timed out after {}s",
                POLL_TIMEOUT.as_secs()
            )));
        }

        let status_resp = client.get(&format!("/jobs/{job_id}")).await?;
        let status = status_resp["job"]["status"].as_u64().unwrap_or(0);

        match status {
            1 | 2 => {
                // Pending or started — continue polling with backoff
                interval = Duration::from_secs_f64(
                    (interval.as_secs_f64() * POLL_BACKOFF).min(POLL_MAX.as_secs_f64()),
                );
            }
            3 => {
                // Success — fetch the query result
                let result_id =
                    status_resp["job"]["query_result_id"]
                        .as_u64()
                        .ok_or_else(|| {
                            Error::Tool("missing query_result_id in job response".to_string())
                        })?;
                return client.get(&format!("/query_results/{result_id}")).await;
            }
            4 => {
                // Failure
                let error_msg = status_resp["job"]["error"]
                    .as_str()
                    .unwrap_or("query execution failed");
                return Err(Error::Tool(error_msg.to_string()));
            }
            5 => {
                return Err(Error::Tool("query execution was cancelled".to_string()));
            }
            _ => {
                return Err(Error::Tool(format!("unknown job status: {status}")));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_query_result_definition_has_max_rows() {
        let defs = definitions();
        let get_def = &defs[0];
        assert_eq!(get_def["name"], "get_query_result");
        let props = &get_def["inputSchema"]["properties"];
        assert!(props.get("max_rows").is_some());
    }

    #[test]
    fn get_query_result_max_rows_not_required() {
        let defs = definitions();
        let get_def = &defs[0];
        let required = get_def["inputSchema"]["required"].as_array().unwrap();
        assert!(!required.contains(&json!("max_rows")));
    }

    #[test]
    fn execute_query_definition_required_fields() {
        let defs = definitions();
        let exec_def = &defs[1];
        assert_eq!(exec_def["name"], "execute_query");
        let required = exec_def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("data_source_id")));
        assert!(required.contains(&json!("query")));
    }

    #[test]
    fn execute_query_definition_optional_params() {
        let defs = definitions();
        let exec_def = &defs[1];
        let props = &exec_def["inputSchema"]["properties"];
        assert!(props.get("max_age").is_some());
        assert!(props.get("max_rows").is_some());
        assert!(props.get("parameters").is_some());
    }

    #[test]
    fn polling_constants_validity() {
        assert!(POLL_TIMEOUT > POLL_MAX);
        assert!(POLL_INITIAL < POLL_MAX);
        assert!(POLL_BACKOFF > 1.0);
    }

    #[test]
    fn get_job_status_definition_required_fields() {
        let defs = definitions();
        let def = defs.iter().find(|d| d["name"] == "get_job_status").unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.contains(&json!("id")));
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 3);
    }

    #[test]
    fn job_response_detected() {
        let with_job = json!({"job": {"id": "abc-123", "status": 1}});
        assert!(with_job.get("job").is_some());

        let immediate = json!({"query_result": {"data": {}}});
        assert!(immediate.get("job").is_none());
    }
}
