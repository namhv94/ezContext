use clap::{Parser, Subcommand};
use ezcontext::ast::AstDispatcher;
use ezcontext::mcp::McpStdioServer;
use ezcontext::token::TokenEstimator;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ezcontext",
    version = "0.1.0",
    about = "ezContext: AST Code-Skeleton & Token Saver MCP Server in Rust (EzLabs)"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Model Context Protocol (MCP) Stdio JSON-RPC 2.0 Server
    Serve,

    /// Extract compact AST code skeleton from a source file
    Skeleton {
        /// File path to analyze
        path: PathBuf,
    },

    /// Generate token outline of a directory
    Outline {
        /// Directory path
        dir: PathBuf,
        #[arg(long, default_value_t = 3)]
        depth: usize,
    },

    /// Analyze token savings and dollar efficiency for a file
    Stats {
        /// File path to analyze
        path: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Serve => {
            // Run stdio loop for MCP clients (Cline, Roo Code, Claude Desktop)
            McpStdioServer::run_loop()?;
        }
        Commands::Skeleton { path } => {
            let content = fs::read_to_string(&path)?;
            let skeleton = AstDispatcher::extract_skeleton(&path, &content)?;
            let savings = TokenEstimator::calculate_savings(&content, &skeleton);

            eprintln!(
                "⚡ [ezContext] {} ➔ {} tokens ({:.1}% saved)",
                savings.original_tokens_est, savings.skeleton_tokens_est, savings.percentage_saved
            );
            println!("{}", skeleton);
        }
        Commands::Outline { dir, depth } => {
            println!(
                "=== ezContext Directory Outline: {} (depth: {}) ===",
                dir.display(),
                depth
            );
            for entry in walkdir::WalkDir::new(&dir)
                .max_depth(depth)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let p = entry.path();
                if p.is_file() {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    let tokens = size.div_ceil(4);
                    println!("- {} (~{} tokens)", p.display(), tokens);
                }
            }
        }
        Commands::Stats { path } => {
            let content = fs::read_to_string(&path)?;
            let skeleton = AstDispatcher::extract_skeleton(&path, &content)?;
            let savings = TokenEstimator::calculate_savings(&content, &skeleton);

            println!("=== ezContext Token Efficiency Report ===");
            println!("File: {}", path.display());
            println!("Original Code Size: {} tokens", savings.original_tokens_est);
            println!("Skeleton AST Size:  {} tokens", savings.skeleton_tokens_est);
            println!("Net Tokens Saved:   {} tokens", savings.tokens_saved);
            println!(
                "Efficiency Gain:    {:.1}% reduction",
                savings.percentage_saved
            );
            println!(
                "Estimated Savings:  ${:.5} USD (at $3/1M input tokens)",
                savings.estimated_cost_saved_usd
            );
        }
    }

    Ok(())
}
