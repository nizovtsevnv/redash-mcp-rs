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
    eprintln!("  --version   Print version information");
    eprintln!("  --help, -h  Print this help message");
    eprintln!();
    eprintln!("Environment:");
    eprintln!("  REDASH_API_KEY   Redash API key (required for --stdio)");
    eprintln!("  REDASH_API_URL   Redash API URL (default: http://localhost:5000/api)");
    eprintln!("  RUST_LOG         Log level filter (default: info)");
}
