use docarmor::batch::AnalysisConfig;
use docarmor::mcp::run_stdio_server;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AnalysisConfig::default();
    eprintln!("[docarmor-mcp] Server starting up on STDIO (protocol version 2024-11-05)...");
    run_stdio_server(&config)?;
    Ok(())
}
