use crate::error::{Error, Result};

/// CLI command to execute.
#[derive(Debug)]
pub enum Command {
    /// Run the MCP server over STDIO transport.
    Stdio,
    /// Run the MCP server over HTTP transport.
    Http,
    /// Print version information and exit.
    Version,
    /// Print usage information and exit.
    Help,
}

/// Parse command-line arguments into a command.
///
/// Accepts: `--stdio`, `--version`, `--help`/`-h`. No args defaults to `Help`.
pub fn parse_args(args: &[String]) -> Result<Command> {
    // Skip the program name (first arg)
    let flags: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    match flags.as_slice() {
        [] => Ok(Command::Help),
        ["--stdio"] => Ok(Command::Stdio),
        ["--http"] => Ok(Command::Http),
        ["--version"] => Ok(Command::Version),
        ["--help" | "-h"] => Ok(Command::Help),
        _ => Err(Error::Config(format!(
            "unknown arguments: {}",
            flags.join(" ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(slice: &[&str]) -> Vec<String> {
        slice.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_stdio_flag() {
        let cmd = parse_args(&args(&["redash-mcp", "--stdio"])).unwrap();
        assert!(matches!(cmd, Command::Stdio));
    }

    #[test]
    fn parse_http_flag() {
        let cmd = parse_args(&args(&["redash-mcp", "--http"])).unwrap();
        assert!(matches!(cmd, Command::Http));
    }

    #[test]
    fn parse_version_flag() {
        let cmd = parse_args(&args(&["redash-mcp", "--version"])).unwrap();
        assert!(matches!(cmd, Command::Version));
    }

    #[test]
    fn parse_help_flag() {
        let cmd = parse_args(&args(&["redash-mcp", "--help"])).unwrap();
        assert!(matches!(cmd, Command::Help));
    }

    #[test]
    fn parse_short_help_flag() {
        let cmd = parse_args(&args(&["redash-mcp", "-h"])).unwrap();
        assert!(matches!(cmd, Command::Help));
    }

    #[test]
    fn parse_no_args_defaults_to_help() {
        let cmd = parse_args(&args(&["redash-mcp"])).unwrap();
        assert!(matches!(cmd, Command::Help));
    }

    #[test]
    fn parse_unknown_flag_errors() {
        let result = parse_args(&args(&["redash-mcp", "--foo"]));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("unknown arguments"));
    }
}
