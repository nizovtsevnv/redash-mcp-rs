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

#### Option 1: Build from source
```bash
git clone https://github.com/nizovtsevnv/redash-mcp-rs.git
cd redash-mcp-rs
cargo build --release
# Binary will be in target/release/redash-mcp
```

#### Option 2: Nix
```bash
nix develop -c cargo build --release
```

### 3. Configure Your AI Agent (STDIO Mode)

JSON configuration for Cursor IDE, Gemini CLI:
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

> **Path Notes:**
> - Use **absolute paths** — relative paths may not work correctly
> - **No spaces** in the executable file path
> - **Windows users**: Use double backslashes `\\` in paths

## Configuration

### Environment Variables

| Variable | Mode | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `REDASH_API_KEY` | STDIO | Yes | — | Redash API key |
| `REDASH_API_URL` | Both | No | `http://localhost:5000/api` | Redash instance API URL |
| `MCP_AUTH_TOKENS` | HTTP | Yes | — | Comma-separated MCP access tokens |
| `HTTP_HOST` | HTTP | No | `127.0.0.1` | Bind address |
| `HTTP_PORT` | HTTP | No | `3000` | Port number (≥ 1024) |
| `HTTP_RATE_LIMIT` | HTTP | No | `60` | Max requests/min per IP |
| `HTTP_SESSION_TIMEOUT` | HTTP | No | `1800` | Session TTL in seconds |
| `HTTP_MAX_BODY_SIZE` | HTTP | No | `1048576` | Max request body in bytes |
| `RUST_LOG` | Both | No | `error` (STDIO) / `info` (HTTP) | Log level |

## Supported Tools (18 planned)

### Data Sources (3)
- `list_data_sources` — List all available data sources
- `get_data_source` — Get data source details
- `get_data_source_schema` — Get table/column schema (key tool for writing queries)

### Queries (7)
- `list_queries` — List saved queries with pagination
- `get_query` — Get query details including SQL
- `search_queries` — Search queries by name/description
- `create_query` — Create a new query
- `update_query` — Update query name, description, or SQL
- `archive_query` — Archive a query
- `list_query_tags` — List all query tags

### Query Execution (2)
- `execute_query` — Execute query and get results
- `get_query_result` — Get latest cached result

### Dashboards (4)
- `list_dashboards` — List dashboards with pagination
- `get_dashboard` — Get dashboard with widgets
- `create_dashboard` — Create a new dashboard
- `list_dashboard_tags` — List all dashboard tags

### Users (2)
- `list_users` — List users
- `get_user` — Get user details

## Architecture

```
┌─────────────────┐    ┌─────────────────────────────┐    ┌─────────────────┐
│   MCP Client    │────│      Transport Layer         │────│   Redash API    │
│ (Claude/Cursor) │    │  STDIO | Streamable HTTP     │    │   (REST/JSON)   │
└─────────────────┘    └─────────────────────────────┘    └─────────────────┘
```

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
├── tools/
│   ├── mod.rs           # Tool registry & dispatcher
│   ├── common.rs        # Shared tool utilities
│   ├── queries.rs       # Query operations
│   ├── data_sources.rs  # Data source operations
│   ├── dashboards.rs    # Dashboard operations
│   └── query_results.rs # Query execution & results
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
- **OR manually**: Rust 1.75+, OpenSSL development libraries

### Build Commands

```bash
nix develop              # Enter dev environment
cargo check              # Quick compilation check
cargo fmt                # Format code
cargo clippy             # Lint
cargo test               # Run tests
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
