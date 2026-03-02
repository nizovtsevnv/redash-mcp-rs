use std::process::ExitCode;

fn main() -> ExitCode {
    // Load .env file if present (before anything else)
    dotenvy::dotenv().ok();

    // Logging to stderr — stdout is reserved for MCP transport
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();

    let args: Vec<String> = std::env::args().collect();

    let command = match redash_mcp_rs::cli::parse_args(&args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error: {e}");
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    match command {
        redash_mcp_rs::cli::Command::Stdio => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create tokio runtime");

            if let Err(e) = rt.block_on(redash_mcp_rs::run_stdio()) {
                tracing::error!("fatal: {e}");
                return ExitCode::FAILURE;
            }
        }
        redash_mcp_rs::cli::Command::Http => {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("failed to create tokio runtime");

            if let Err(e) = rt.block_on(redash_mcp_rs::run_http()) {
                tracing::error!("fatal: {e}");
                return ExitCode::FAILURE;
            }
        }
        redash_mcp_rs::cli::Command::Version => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        }
        redash_mcp_rs::cli::Command::Help => {
            print_usage();
        }
    }

    ExitCode::SUCCESS
}

fn print_usage() {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    eprintln!("{name} v{version} — MCP server for Redash");
    eprintln!();
    eprintln!("Usage: redash-mcp [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  --stdio     Run MCP server over STDIO transport");
    eprintln!("  --http      Run MCP server over HTTP transport");
    eprintln!("  --version   Print version information");
    eprintln!("  --help, -h  Print this help message");
    eprintln!();
    eprintln!("Environment:");
    eprintln!("  REDASH_API_KEY        Redash API key (required for --stdio)");
    eprintln!("  REDASH_API_URL        Redash API URL (default: http://localhost:5000/api)");
    eprintln!("  MCP_AUTH_TOKENS       Comma-separated auth tokens (required for --http)");
    eprintln!("  MCP_HOST              HTTP server host (default: 127.0.0.1)");
    eprintln!("  MCP_PORT              HTTP server port (default: 3000)");
    eprintln!("  MCP_MAX_BODY_SIZE     Max request body size in bytes (default: 1048576)");
    eprintln!("  MCP_SESSION_TIMEOUT   Session timeout in seconds (default: 1800)");
    eprintln!("  MCP_RATE_LIMIT        Max requests per minute per IP (default: 60)");
    eprintln!("  RUST_LOG              Log level filter (default: info)");
}
