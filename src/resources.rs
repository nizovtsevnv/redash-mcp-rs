use crate::error::{Error, Result};
use crate::redash::RedashClient;
use serde_json::Value;

/// Return the list of resource templates supported by the server.
pub fn resource_templates() -> Vec<Value> {
    vec![
        serde_json::json!({
            "uriTemplate": "redash://datasource/{id}/schema",
            "name": "Data source schema",
            "description": "Schema (tables and columns) for a Redash data source",
            "mimeType": "application/json"
        }),
        serde_json::json!({
            "uriTemplate": "redash://query/{id}",
            "name": "Query details",
            "description": "SQL query text and metadata for a Redash query",
            "mimeType": "application/json"
        }),
        serde_json::json!({
            "uriTemplate": "redash://dashboard/{slug}",
            "name": "Dashboard details",
            "description": "Dashboard metadata and widgets",
            "mimeType": "application/json"
        }),
    ]
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
    let data = if uri.starts_with("redash://datasource/") {
        let id = parse_datasource_schema_uri(uri)?;
        client.get(&format!("/data_sources/{id}/schema")).await?
    } else if uri.starts_with("redash://query/") {
        let id = parse_query_uri(uri)?;
        client.get(&format!("/queries/{id}")).await?
    } else if uri.starts_with("redash://dashboard/") {
        let slug = parse_dashboard_uri(uri)?;
        client.get(&format!("/dashboards/{slug}")).await?
    } else {
        return Err(Error::Tool(format!("unsupported resource URI: {uri}")));
    };

    let text = serde_json::to_string_pretty(&data).unwrap_or_else(|_| data.to_string());

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

/// Parse a `redash://query/{id}` URI and extract the query ID.
fn parse_query_uri(uri: &str) -> Result<u64> {
    let id_str = uri
        .strip_prefix("redash://query/")
        .ok_or_else(|| Error::Tool(format!("unsupported resource URI: {uri}")))?;

    id_str
        .parse::<u64>()
        .map_err(|_| Error::Tool(format!("invalid query ID in URI: {uri}")))
}

/// Parse a `redash://dashboard/{slug}` URI and extract the dashboard slug.
fn parse_dashboard_uri(uri: &str) -> Result<String> {
    let slug = uri
        .strip_prefix("redash://dashboard/")
        .ok_or_else(|| Error::Tool(format!("unsupported resource URI: {uri}")))?;

    if slug.is_empty() {
        return Err(Error::Tool(format!("empty dashboard slug in URI: {uri}")));
    }

    Ok(slug.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resource_templates_count() {
        let templates = resource_templates();
        assert_eq!(templates.len(), 3);
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

    #[test]
    fn parse_valid_query_uri() {
        let id = parse_query_uri("redash://query/42").unwrap();
        assert_eq!(id, 42);
    }

    #[test]
    fn parse_invalid_query_uri() {
        let err = parse_query_uri("redash://query/abc").unwrap_err();
        assert!(err.to_string().contains("invalid query ID"));
    }

    #[test]
    fn parse_valid_dashboard_uri() {
        let slug = parse_dashboard_uri("redash://dashboard/my-dashboard").unwrap();
        assert_eq!(slug, "my-dashboard");
    }

    #[test]
    fn parse_empty_dashboard_slug() {
        let err = parse_dashboard_uri("redash://dashboard/").unwrap_err();
        assert!(err.to_string().contains("empty dashboard slug"));
    }

    #[test]
    fn all_templates_have_required_fields() {
        for t in resource_templates() {
            assert!(t.get("uriTemplate").is_some(), "missing uriTemplate: {t}");
            assert!(t.get("name").is_some(), "missing name: {t}");
            assert!(t.get("description").is_some(), "missing description: {t}");
            assert!(t.get("mimeType").is_some(), "missing mimeType: {t}");
        }
    }
}
