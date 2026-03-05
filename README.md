# Redash MCP Server

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

MCP (Model Context Protocol) server for Redash API integration with focus on **simplicity**, **performance**, and **reliability**.

Supports two transport modes:
- **STDIO** — single-user, for direct integration with MCP clients (Cursor IDE, Claude Desktop, etc.)
- **HTTP** — multi-user Streamable HTTP transport with authentication, rate limiting, sessions, SSE, and CORS

## Quick Start

### 1. Get Your Redash API Key

Go to your Redash instance: **Settings → Account → API Key**

### 2. Download & Install

#### Option 1: Download pre-built binary

Download the latest release from [GitHub Releases](https://github.com/nizovtsevnv/redash-mcp-rs/releases):

| Platform | Architecture | File |
|----------|-------------|------|
| Linux | x86_64 | `redash-mcp-x86_64-unknown-linux-musl.tar.gz` |
| Linux | aarch64 | `redash-mcp-aarch64-unknown-linux-musl.tar.gz` |
| macOS | x86_64 | `redash-mcp-x86_64-apple-darwin.tar.gz` |
| macOS | aarch64 (Apple Silicon) | `redash-mcp-aarch64-apple-darwin.tar.gz` |

Linux binaries are statically linked (musl) and require no dependencies.

```bash
tar xzf redash-mcp-*.tar.gz
chmod +x redash-mcp
```

#### Option 2: Build from source
```bash
git clone https://github.com/nizovtsevnv/redash-mcp-rs.git
cd redash-mcp-rs
cargo build --release
# Binary will be in target/release/redash-mcp
```

#### Option 3: Docker
```bash
docker build -t redash-mcp-rs .
docker run -e REDASH_API_KEY=your-key -e REDASH_API_URL=http://redash:5000/api redash-mcp-rs
```

#### Option 4: Nix
```bash
nix develop -c cargo build --release
```

### 3. Configure Your AI Agent (STDIO Mode)

#### Cursor IDE, Gemini CLI
```json
{
  "mcpServers": {
    "Redash analytics": {
      "command": "/path/to/redash-mcp",
      "env": {
        "REDASH_API_KEY": "your-api-key-here",
        "REDASH_API_URL": "http://your-redash-instance:5000/api"
      }
    }
  }
}
```

#### Claude Code
```bash
claude mcp add redash-analytics \
  -e REDASH_API_KEY=your-api-key-here \
  -e REDASH_API_URL=http://your-redash-instance:5000/api \
  -- /path/to/redash-mcp
```

> **Path Notes:**
> - Use **absolute paths** — relative paths may not work correctly
> - **No spaces** in the executable file path
> - **Windows users**: Use double backslashes `\\` in paths

### 4. Run in HTTP Mode (multi-user)

```bash
export REDASH_API_URL="http://your-redash-instance:5000/api"
export MCP_AUTH_TOKENS="token1,token2"
redash-mcp --http
```

Clients authenticate with `Authorization: Bearer <token>` and pass their Redash API key via `X-Redash-API-Key` header.

## Configuration

### Environment Variables

| Variable | Mode | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `REDASH_API_KEY` | STDIO | Yes | — | Redash API key |
| `REDASH_API_URL` | Both | No | `http://localhost:5000/api` | Redash instance API URL |
| `REDASH_TIMEOUT` | Both | No | `30` | Request timeout in seconds (1–300) |
| `REDASH_MAX_RETRIES` | Both | No | `2` | Max retries on network errors (0–5) |
| `MCP_AUTH_TOKENS` | HTTP | Yes | — | Comma-separated MCP access tokens |
| `MCP_HOST` | HTTP | No | `127.0.0.1` | Bind address |
| `MCP_PORT` | HTTP | No | `3000` | Port number (>= 1024) |
| `MCP_RATE_LIMIT` | HTTP | No | `60` | Max requests/min per IP |
| `MCP_SESSION_TIMEOUT` | HTTP | No | `1800` | Session TTL in seconds |
| `MCP_MAX_BODY_SIZE` | HTTP | No | `1048576` | Max request body in bytes |
| `RUST_LOG` | Both | No | `error`/`info` | Log level (STDIO/HTTP) |

## Tools (60)

### Data Sources (6)
- `list_data_sources` — List all available data sources
- `get_data_source` — Get data source details
- `get_data_source_schema` — Get table/column schema
- `test_data_source` — Test data source connection
- `list_data_source_types` — List available data source types
- `pause_data_source` — Pause a data source

### Queries (12)
- `list_queries` — List saved queries with pagination
- `get_query` — Get query details including SQL
- `search_queries` — Search queries by name/description
- `create_query` — Create a new query
- `update_query` — Update query name, description, or SQL
- `archive_query` — Archive a query
- `refresh_query` — Force refresh query results
- `fork_query` — Fork a query
- `list_query_tags` — List all query tags
- `list_my_queries` — List my queries
- `list_recent_queries` — List recent queries
- `list_archived_queries` — List archived queries

### Query Execution (3)
- `execute_query` — Execute query and get results
- `get_query_result` — Get latest cached result
- `get_job_status` — Get background job status

### Dashboards (10)
- `list_dashboards` — List dashboards with pagination
- `get_dashboard` — Get dashboard with widgets
- `create_dashboard` — Create a new dashboard
- `update_dashboard` — Update dashboard name or tags
- `archive_dashboard` — Archive a dashboard
- `list_dashboard_tags` — List all dashboard tags
- `share_dashboard` — Enable public dashboard access
- `unshare_dashboard` — Disable public dashboard access
- `list_my_dashboards` — List my dashboards
- `fork_dashboard` — Fork a dashboard

### Users (2)
- `list_users` — List users
- `get_user` — Get user details

### Visualizations (3)
- `create_visualization` — Create a visualization for a query
- `update_visualization` — Update a visualization
- `delete_visualization` — Delete a visualization

### Widgets (3)
- `add_widget` — Add a widget to a dashboard
- `update_widget` — Update a widget
- `remove_widget` — Remove a widget

### Alerts (9)
- `list_alerts` — List all alerts
- `get_alert` — Get alert details
- `create_alert` — Create an alert
- `update_alert` — Update an alert
- `delete_alert` — Delete an alert
- `mute_alert` — Mute an alert
- `list_alert_subscriptions` — List alert subscriptions
- `create_alert_subscription` — Subscribe a destination to an alert
- `delete_alert_subscription` — Remove a subscription

### Query Snippets (5)
- `list_query_snippets` — List query snippets
- `get_query_snippet` — Get snippet details
- `create_query_snippet` — Create a query snippet
- `update_query_snippet` — Update a query snippet
- `delete_query_snippet` — Delete a query snippet

### Favorites (6)
- `favorite_query` / `unfavorite_query` — Add/remove query from favorites
- `favorite_dashboard` / `unfavorite_dashboard` — Add/remove dashboard from favorites
- `list_favorite_queries` — List favorite queries
- `list_favorite_dashboards` — List favorite dashboards

### Destinations (1)
- `list_destinations` — List notification destinations

## Resources (3 templates)

| URI Template | Description |
|-------------|-------------|
| `redash://datasource/{id}/schema` | Data source schema (tables and columns) |
| `redash://query/{id}` | Query SQL and metadata |
| `redash://dashboard/{slug}` | Dashboard with widgets |

## Prompts (5)

| Prompt | Required Args | Description |
|--------|--------------|-------------|
| `explore_data` | `data_source_id` | Get schema, write exploratory query, execute, analyze |
| `build_dashboard` | `dashboard_name` | Find queries, create dashboard, add visualizations and widgets |
| `setup_alert` | `query_id` | Check query results, configure threshold, create alert |
| `optimize_query` | `query_id` | Analyze SQL, check schema, suggest optimizations, fork |
| `monitor_system` | — | Check data sources, connectivity, recent queries, alerts |

## Architecture

```
+------------------+    +------------------------------+    +------------------+
|   MCP Client     |----|      Transport Layer          |----|   Redash API     |
| (Claude/Cursor)  |    |  STDIO | Streamable HTTP      |    |   (REST/JSON)    |
+------------------+    +------------------------------+    +------------------+
```

### HTTP Request Pipeline

```
Request -> Router -> Auth (Bearer) -> Rate Limit -> Session -> MCP Dispatch -> Response
```

Endpoints: `POST /mcp` (request), `GET /mcp` (SSE), `DELETE /mcp` (close session), `GET /health`

### Source Layout

```
src/
├── main.rs              # Entry point, CLI, runtime setup
├── lib.rs               # Public API: run_stdio(), run_http()
├── cli.rs               # CLI argument parsing
├── config.rs            # Environment-based configuration
├── error.rs             # Centralized error types
├── mcp.rs               # MCP JSON-RPC 2.0 protocol handler
├── redash.rs            # Redash API HTTP client
├── logging.rs           # MCP Logging (LogLevel, notifications)
├── progress.rs          # MCP Progress tracking
├── resources.rs         # MCP Resources (3 URI templates)
├── prompts.rs           # MCP Prompts (5 guided workflows)
├── tools/
│   ├── mod.rs           # Tool registry & dispatcher (60 tools)
│   ├── common.rs        # Shared tool utilities
│   ├── queries.rs       # Query operations
│   ├── data_sources.rs  # Data source operations
│   ├── dashboards.rs    # Dashboard operations
│   ├── query_results.rs # Query execution & results
│   ├── users.rs         # User operations
│   ├── visualizations.rs # Visualization operations
│   ├── widgets.rs       # Widget operations
│   ├── alerts.rs        # Alert operations
│   ├── snippets.rs      # Query snippet operations
│   ├── favorites.rs     # Favorite operations
│   └── destinations.rs  # Notification destinations
└── http/
    ├── server.rs        # TCP listener, graceful shutdown
    ├── router.rs        # Request routing
    ├── handler.rs       # MCP endpoint handlers
    ├── auth.rs          # Token validation, rate limiting
    ├── session.rs       # Session management
    ├── sse.rs           # Server-Sent Events
    └── ...
```

## Development

### Requirements
- **Nix** (recommended) — handles all dependencies automatically
- **OR manually**: Rust 1.75+

### Build Commands

```bash
nix develop              # Enter dev environment
cargo check              # Quick compilation check
cargo fmt                # Format code
cargo clippy             # Lint
cargo test               # Run tests (350 tests)
cargo build --release    # Release build
```

## Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Make changes with tests
4. Ensure all checks pass: `cargo fmt && cargo clippy && cargo test`
5. Submit pull request

## License

MIT License — see [LICENSE](LICENSE) file for details.
