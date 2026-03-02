#![deny(unsafe_code)]
#![warn(clippy::all)]

pub mod cli;
pub mod config;
pub mod error;
pub mod http;
pub mod mcp;
pub mod prompts;
pub mod redash;
pub mod resources;
pub mod tools;

use error::{Error, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Run the MCP server over STDIO transport.
///
/// Reads JSON-RPC messages line-by-line from stdin, dispatches via
/// `mcp::handle_message`, and writes responses to stdout. Logging
/// goes to stderr to keep stdout clean for the MCP protocol.
pub async fn run_stdio() -> Result<()> {
    let config = config::load_stdio_config()?;
    let client = redash::RedashClient::new(config.api_url, config.api_key);

    tracing::info!("MCP server started in STDIO mode");

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .map_err(|e| Error::Transport(format!("failed to read stdin: {e}")))?;

        if bytes_read == 0 {
            tracing::info!("stdin closed, shutting down");
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        tracing::debug!("received: {trimmed}");

        match mcp::handle_message(trimmed, &client).await? {
            Some(response) => {
                tracing::debug!("sending: {response}");
                stdout
                    .write_all(response.as_bytes())
                    .await
                    .map_err(|e| Error::Transport(format!("failed to write stdout: {e}")))?;
                stdout
                    .write_all(b"\n")
                    .await
                    .map_err(|e| Error::Transport(format!("failed to write stdout: {e}")))?;
                stdout
                    .flush()
                    .await
                    .map_err(|e| Error::Transport(format!("failed to flush stdout: {e}")))?;
            }
            None => {
                tracing::debug!("notification processed, no response");
            }
        }
    }

    Ok(())
}
