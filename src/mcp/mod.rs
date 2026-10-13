pub mod protocol;

use crate::ast::AstDispatcher;
use crate::token::TokenEstimator;
use protocol::{JsonRpcRequest, JsonRpcResponse};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use walkdir::WalkDir;

pub struct McpStdioServer;

impl McpStdioServer {
    pub fn run_loop() -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line_res in stdin.lock().lines() {
            let line = line_res?;
            if line.trim().is_empty() {
                continue;
            }

            let req: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                    });
                    writeln!(stdout, "{}", err_resp)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = Self::handle_request(&req);
            let out_str = serde_json::to_string(&response).unwrap_or_default();
            writeln!(stdout, "{}", out_str)?;
            stdout.flush()?;
        }

        Ok(())
    }

    pub fn handle_request(req: &JsonRpcRequest) -> JsonRpcResponse {
        let id = req.id.clone();
        match req.method.as_str() {
            "initialize" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "serverInfo": { "name": "ezcontext-mcp-rust", "version": "0.1.0" },
                    "capabilities": {
                        "tools": { "listChanged": true }
                    }
                })),
                error: None,
            },
            "tools/list" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "get_skeleton",
                            "description": "Extract compact AST code skeleton (structs, functions, interfaces) omitting heavy function bodies to save 80-90% token context.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "Absolute or relative file path" }
                                },
                                "required": ["path"]
                            }
                        },
                        {
                            "name": "get_outline",
                            "description": "Scan a directory and generate high-level architectural outline with estimated token counts per module.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "dir_path": { "type": "string", "description": "Target directory path" },
                                    "max_depth": { "type": "integer", "description": "Maximum traversal depth (default: 3)" }
                                },
                                "required": ["dir_path"]
                            }
                        },
                        {
                            "name": "read_symbol",
                            "description": "Drill down and read the exact implementation of a specific function, class, or struct without loading the entire file.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "File path" },
                                    "symbol_name": { "type": "string", "description": "Name of the function, struct, or class to inspect" }
                                },
                                "required": ["path", "symbol_name"]
                            }
                        },
                        {
                            "name": "calculate_savings",
                            "description": "Calculate token count and estimated dollar savings achieved by skeleton extraction.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string", "description": "File path to analyze" }
                                },
                                "required": ["path"]
                            }
                        }
                    ]
                })),
                error: None,
            },
            "tools/call" => {
                let name = req
                    .params
                    .get("name")
                    .and_then(|n| n.as_str())
                    .unwrap_or("");
                let args = req.params.get("arguments").cloned().unwrap_or(json!({}));
                let tool_res = Self::execute_tool(name, &args);

                match tool_res {
                    Ok(text) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [{ "type": "text", "text": text }],
                            "isError": false
                        })),
                        error: None,
                    },
                    Err(err) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id,
                        result: Some(json!({
                            "content": [{ "type": "text", "text": format!("Error: {}", err) }],
                            "isError": true
                        })),
                        error: None,
                    },
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(
                    json!({ "code": -32601, "message": format!("Method '{}' not found", req.method) }),
                ),
            },
        }
    }

    fn execute_tool(name: &str, args: &Value) -> Result<String, String> {
        match name {
            "get_skeleton" => {
                let path_str = args
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("Missing 'path' argument")?;
                let content = fs::read_to_string(path_str)
                    .map_err(|e| format!("Cannot read file '{}': {}", path_str, e))?;
                let skeleton = AstDispatcher::extract_skeleton(path_str, &content)?;
                let savings = TokenEstimator::calculate_savings(&content, &skeleton);

                Ok(format!(
                    "// [ezContext AST Skeleton: {} (Saved {}% tokens | {} -> {} tokens)]\n\n{}",
                    path_str,
                    savings.percentage_saved as u64,
                    savings.original_tokens_est,
                    savings.skeleton_tokens_est,
                    skeleton
                ))
            }
            "get_outline" => {
                let dir_str = args
                    .get("dir_path")
                    .and_then(|d| d.as_str())
                    .ok_or("Missing 'dir_path' argument")?;
                let max_depth =
                    args.get("max_depth").and_then(|d| d.as_u64()).unwrap_or(3) as usize;

                let mut outline_entries = Vec::new();
                for entry in WalkDir::new(dir_str)
                    .max_depth(max_depth)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() {
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let token_est = size.div_ceil(4) as usize;
                        let rel = path.strip_prefix(dir_str).unwrap_or(path);
                        outline_entries.push(format!(
                            "- {} (~{} tokens)",
                            rel.display(),
                            token_est
                        ));
                    }
                }
                Ok(format!(
                    "=== Architecture Outline: {} ===\nFound {} source files:\n\n{}",
                    dir_str,
                    outline_entries.len(),
                    outline_entries.join("\n")
                ))
            }
            "read_symbol" => {
                let path_str = args
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("Missing 'path' argument")?;
                let symbol = args
                    .get("symbol_name")
                    .and_then(|s| s.as_str())
                    .ok_or("Missing 'symbol_name' argument")?;
                let content = fs::read_to_string(path_str)
                    .map_err(|e| format!("Cannot read file '{}': {}", path_str, e))?;
                let symbol_def = AstDispatcher::read_symbol(&content, symbol)?;
                Ok(format!(
                    "// [ezContext Symbol: {} from {}]\n\n{}",
                    symbol, path_str, symbol_def
                ))
            }
            "calculate_savings" => {
                let path_str = args
                    .get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("Missing 'path' argument")?;
                let content = fs::read_to_string(path_str)
                    .map_err(|e| format!("Cannot read file '{}': {}", path_str, e))?;
                let skeleton = AstDispatcher::extract_skeleton(path_str, &content)?;
                let savings = TokenEstimator::calculate_savings(&content, &skeleton);

                Ok(format!(
                    "ezContext Token Savings Report for '{}':\n- Original Tokens: {}\n- Skeleton Tokens: {}\n- Tokens Saved: {} ({:.1}%)\n- Cost Savings (Claude 3.5 Sonnet tier): ${:.5} USD",
                    path_str,
                    savings.original_tokens_est,
                    savings.skeleton_tokens_est,
                    savings.tokens_saved,
                    savings.percentage_saved,
                    savings.estimated_cost_saved_usd
                ))
            }
            _ => Err(format!("Unknown tool '{}'", name)),
        }
    }
}
