use crate::error::{Error, Result};
use crate::redash::RedashClient;
use serde_json::Value;

/// Return the list of resource templates supported by the server.
pub fn resource_templates() -> Vec<Value> {
    vec![serde_json::json!({
        "uriTemplate": "redash://datasource/{id}/schema",
        "name": "Data source schema",
        "description": "Schema (tables and columns) for a Redash data source",
        "mimeType": "application/json"
    })]
}

/// Return the static resource list (empty — we use templates only).
pub fn resource_list() -> Vec<Value> {
    vec![]
}

/// Read a resource by URI.
///
/// Currently supports `redash://datasource/{id}/schema` which fetches
/// the schema from the Redash API and wraps it in MCP resource content format.
pub async fn read_resource(uri: &str, client: &RedashClient) -> Result<Value> {
    let id = parse_datasource_schema_uri(uri)?;
    let schema = client.get(&format!("/data_sources/{id}/schema")).await?;
    let text = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| schema.to_string());

    Ok(serde_json::json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": text
        }]
    }))
}

/// Parse a `redash://datasource/{id}/schema` URI and extract the data source ID.
fn parse_datasource_schema_uri(uri: &str) -> Result<u64> {
    let path = uri
        .strip_prefix("redash://datasource/")
        .ok_or_else(|| Error::Tool(format!("unsupported resource URI: {uri}")))?;

    let id_str = path
        .strip_suffix("/schema")
        .ok_or_else(|| Error::Tool(format!("unsupported resource URI: {uri}")))?;

    id_str
        .parse::<u64>()
        .map_err(|_| Error::Tool(format!("invalid data source ID in URI: {uri}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resource_templates_count() {
        let templates = resource_templates();
        assert_eq!(templates.len(), 1);
    }

    #[test]
    fn resource_template_has_required_fields() {
        let templates = resource_templates();
        let t = &templates[0];
        assert!(t.get("uriTemplate").is_some());
        assert!(t.get("name").is_some());
        assert!(t.get("description").is_some());
        assert!(t.get("mimeType").is_some());
    }

    #[test]
    fn resource_list_is_empty() {
        assert!(resource_list().is_empty());
    }

    #[test]
    fn parse_valid_datasource_uri() {
        let id = parse_datasource_schema_uri("redash://datasource/42/schema").unwrap();
        assert_eq!(id, 42);
    }

    #[test]
    fn parse_datasource_uri_id_1() {
        let id = parse_datasource_schema_uri("redash://datasource/1/schema").unwrap();
        assert_eq!(id, 1);
    }

    #[test]
    fn parse_invalid_prefix() {
        let err = parse_datasource_schema_uri("http://datasource/1/schema").unwrap_err();
        assert!(err.to_string().contains("unsupported resource URI"));
    }

    #[test]
    fn parse_invalid_suffix() {
        let err = parse_datasource_schema_uri("redash://datasource/1/tables").unwrap_err();
        assert!(err.to_string().contains("unsupported resource URI"));
    }

    #[test]
    fn parse_invalid_id() {
        let err = parse_datasource_schema_uri("redash://datasource/abc/schema").unwrap_err();
        assert!(err.to_string().contains("invalid data source ID"));
    }

    #[test]
    fn parse_empty_id() {
        let err = parse_datasource_schema_uri("redash://datasource//schema").unwrap_err();
        assert!(err.to_string().contains("invalid data source ID"));
    }

    #[test]
    fn resource_template_uri_pattern() {
        let templates = resource_templates();
        assert_eq!(
            templates[0]["uriTemplate"],
            json!("redash://datasource/{id}/schema")
        );
    }
}
