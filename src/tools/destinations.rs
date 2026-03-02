use crate::error::Result;
use crate::redash::RedashClient;
use crate::tools::common::format_tool_result;
use serde_json::Value;

/// Tool definitions for alert destination tools.
pub fn definitions() -> Vec<Value> {
    vec![serde_json::json!({
        "name": "list_destinations",
        "description": "List all alert notification destinations",
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
    })]
}

/// List all alert notification destinations.
pub async fn list(client: &RedashClient) -> Result<Value> {
    let data = client.get("/destinations").await?;
    Ok(format_tool_result(&data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_destinations_definition_no_required() {
        let defs = definitions();
        let def = defs
            .iter()
            .find(|d| d["name"] == "list_destinations")
            .unwrap();
        let required = def["inputSchema"]["required"].as_array().unwrap();
        assert!(required.is_empty());
    }

    #[test]
    fn definitions_count() {
        assert_eq!(definitions().len(), 1);
    }
}
