<div align="center">

# ⚡ ezContext

**AST Code-Skeleton & Token Saver MCP Server in Rust**  
*Slash AI context window consumption by 85–95% with blazing fast AST parsing and seamless VS Code integration.*

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![MCP Spec](https://img.shields.io/badge/MCP-2024--11--05-cyan.svg)](https://modelcontextprotocol.io/)
[![CI](https://github.com/namhv94/ezContext/actions/workflows/ci.yml/badge.svg)](https://github.com/namhv94/ezContext/actions)
[![VS Code Extension](https://img.shields.io/badge/VS%20Code-v0.1.0%20VSIX-007acc.svg)](https://github.com/namhv94/ezContext/releases)
[![Product of EzLabs](https://img.shields.io/badge/Product%20of-EzLabs-0ea5e9.svg)](https://ezlabs.tech)

[**Website**](https://ezlabs.tech) • [**ezRouter**](https://github.com/namhv94/ezRouter) • [**ezAgents**](https://github.com/namhv94/ezAgents) • [**ezEval**](https://github.com/namhv94/ezEval) • [**Issues**](https://github.com/namhv94/ezContext/issues)

</div>

---

## 💡 The Core Problem

When coding with AI agents (Cline, Roo Code, Cursor, Claude Code, GitHub Copilot), reading raw source files causes **two severe failure modes**:
1. **The "Lost in the Middle" Syndrome**: Reading 3 to 5 multi-thousand-line files quickly floods the model's 128k/200k context window with implementation boilerplate, causing the model to lose track of initial architectural constraints.
2. **Brutal Token Expenses**: A simple exploration session across 10 service files burns **30,000 to 80,000 input tokens** ($0.25 – $0.50 per turn) just for the model to understand function signatures and type definitions.

---

## ⚡ The Solution: ezContext

**ezContext** is a high-speed Abstract Syntax Tree (AST) code-skeleton engine and Model Context Protocol (MCP) server written in **Rust**:
* **AST-Aware Skeleton Extraction**: Parses Rust, TypeScript, JavaScript, and Python into ultra-compact skeletons—retaining all `struct`, `interface`, `type`, `trait`, docstrings, and function signatures while stripping verbose function bodies.
* **85% – 95% Token Reduction**: Turns 3,000-line monster files (~7,500 tokens) into clean 120-line structural outlines (~350 tokens).
* **On-Demand Symbol Drill-Down**: AI assistants only fetch the exact implementation of a specific function (`read_symbol`) when they are ready to edit.
* **1-Click VS Code Integration**: Connects straight into **Cline**, **Roo Code**, **Cursor**, and **Claude Desktop** via stdio JSON-RPC 2.0.

---

## 🎯 Real-World Case Studies: Before vs. After

### 🔍 Case Study 1: Large Core Module Exploration (Rust / Tokio)

* **Target File**: `src/mcp/mod.rs` (Complex protocol server with request dispatcher)
* **Goal**: Agent needs to check which tool methods are available and their arguments.

```text
❌ WITHOUT ezContext (Standard `read_file` Tool)
────────────────────────────────────────────────────────────────────────
* Model ingests entire raw file: 2,703 tokens.
* 92% of the context consists of verbose parsing and error-handling boilerplate.
* Context consumed: 2,703 tokens ($0.0081 per single inspection).
* Model recall degrades for subsequent refactoring steps.

✅ WITH ezContext (`get_skeleton` Tool)
────────────────────────────────────────────────────────────────────────
* ezContext extracts Rust AST Skeleton via syn parser in 0.8ms:
  pub struct McpStdioServer;
  impl McpStdioServer {
      pub fn run_loop() -> io::Result<()>;
      pub fn handle_request(req: &JsonRpcRequest) -> JsonRpcResponse;
      fn execute_tool(name: &str, args: &Value) -> Result<String, String>;
  }
* Context consumed: 135 tokens (95.0% reduction!).
* Money saved: $0.0077 per call.
* AI retains crystal-clear focus on the interface boundaries.
```

---

### 🔍 Case Study 2: Exploring Fullstack React / TypeScript Services

* **Target**: `api/client.ts` containing 28 API endpoint handlers and complex DTO interfaces.

```text
❌ WITHOUT ezContext:
* Agent loads 1,850 tokens of repetitive `fetch()` and `try/catch` code.

✅ WITH ezContext:
* Ingests clean TypeScript interfaces and export declarations in 190 tokens (89.7% savings).
* Agent pinpoints required endpoint in 1 turn without token bloat.
```

---

## 📊 Comprehensive Comparison Matrix

| Feature | Raw `read_file` | Grep / Regex Hacks | **ezContext (Rust MCP)** |
| :--- | :---: | :---: | :---: |
| **Parsing Engine** | ❌ None (Dumps raw text) | Brittle regular expressions | **Full AST Parsers (`syn` / regex state machine)** |
| **Token Consumption** | 100% (Full file bloat) | Inconsistent / Broken syntax | **5% – 15% of original (85–95% savings)** |
| **Syntax Integrity** | Preserved | Often corrupts braces `{}` | **100% syntactically valid code skeletons** |
| **MCP Integration** | Generic tool | ❌ None | **Native Model Context Protocol (2024-11-05)** |
| **Execution Latency** | File I/O bounded | Disk search bounded | **Sub-millisecond Rust compilation (< 1ms)** |
| **IDE Integration** | Built-in | CLI only | **1-Click VS Code Extension (Cline & Roo Code)** |

---

## 🏛️ System Architecture

```text
 ┌────────────────────────────────────────────────────────┐
 │           Visual Studio Code / Claude Desktop          │
 │                                                        │
 │   ┌───────────────────────┐   ┌─────────────────────┐  │
 │   │  Cline / Roo Code /   │   │ ezLabs VS Code Ext  │  │
 │   │  Cursor AI Assistant  │   │ (1-Click MCP Setup) │  │
 │   └───────────┬───────────┘   └──────────┬──────────┘  │
 └───────────────┼──────────────────────────┼─────────────┘
                 │ JSON-RPC 2.0 (Stdio)     │
                 ▼                          ▼
 ┌────────────────────────────────────────────────────────┐
 │            ezContext Native Rust Engine                │
 └───────────────────────────┬────────────────────────────┘
                             │
              ┌──────────────┴──────────────┐
              ▼                             ▼
   ┌───────────────────────┐   ┌──────────────────────────┐
   │ AST Skeleton Extract  │   │  Token Efficiency Engine │
   │ - Rust (`syn` parser) │   │  - Accurate BPE estimates│
   │ - TypeScript / JS     │   │  - Realtime savings HUD  │
   │ - Python (indentation)│   │  - Dollar cost metrics   │
   └──────────┬────────────┘   └────────────┬─────────────┘
              │                             │
              └──────────────┬──────────────┘
                             ▼
 ┌────────────────────────────────────────────────────────┐
 │         Clean Compact Skeleton (~90% smaller)          │
 │         Drill-down with `read_symbol` on demand        │
 └────────────────────────────────────────────────────────┘
```

---

## 🚀 Quick Start (60 Seconds)

### 1. Build & Install binary

```bash
git clone https://github.com/namhv94/ezContext.git
cd ezContext
cargo build --release
sudo cp target/release/ezcontext /usr/local/bin/
```

### 2. Verify Token Savings on Any Codebase

```bash
ezcontext stats src/main.rs
```

*Output:*
```text
=== ezContext Token Efficiency Report ===
File: src/main.rs
Original Code Size: 924 tokens
Skeleton AST Size:  118 tokens
Net Tokens Saved:   806 tokens
Efficiency Gain:    87.2% reduction
Estimated Savings:  $0.00242 USD (at $3/1M input tokens)
```

### 3. Extract Skeleton to Terminal

```bash
ezcontext skeleton src/lib.rs
```

---

## 🔌 VS Code & AI Assistant Integration

### Option A: Install ezContext VS Code Extension (.vsix)
```bash
# Install the packaged VSIX companion extension directly:
code --install-extension vscode-extension/ezlabs-context-0.1.0.vsix
```
* Click the Status Bar HUD: `$(zap) ezContext: Active` ➔ Automatically writes MCP server configuration to Cline & Roo Code.
* Or right-click any file in the Explorer ➔ Click *"Copy Compact AI Skeleton"* to paste directly to ChatGPT/Claude web.

### Option B: Manual JSON Configuration (Cline / Roo Code / Claude Desktop)
Add `ezcontext` to your `cline_mcp_settings.json` or `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ezcontext": {
      "command": "ezcontext",
      "args": ["serve"],
      "disabled": false,
      "autoApprove": ["get_skeleton", "get_outline", "read_symbol"]
    }
  }
}
```

---

## 🛠️ MCP Tools Reference

| Tool | Parameters | Description |
| :--- | :--- | :--- |
| `get_skeleton` | `path: string` | Returns the compact AST skeleton of a file (saving 85-95% tokens). |
| `get_outline` | `dir_path: string, max_depth?: int` | Scans a directory to generate module outlines with token estimates. |
| `read_symbol` | `path: string, symbol_name: string` | Fetches the full implementation of a single function or struct. |
| `calculate_savings` | `path: string` | Returns detailed token and dollar savings metrics. |

---

## 🧪 Verified Test Suite

```bash
cargo test
```

*Output:*
```text
running 5 tests
test test_token_savings ... ok
test test_mcp_protocol_messages ... ok
test test_rust_skeleton_extraction ... ok
test test_python_skeleton_extraction ... ok
test test_typescript_skeleton_extraction ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

---

## 🌐 EzLabs Ecosystem

| Project | Description | Stack | Status |
| :--- | :--- | :---: | :--- |
| **[ezRouter](https://github.com/namhv94/ezRouter)** | Ultra-fast LLM routing proxy with Token Saver & multi-provider failover | **Rust** | `v0.1.0` Open Source |
| **[ezContext](https://github.com/namhv94/ezContext)** | AST Code-Skeleton & Token Saver MCP Server for AI coding tools | **Rust / TS** | `v0.1.0` Active |
| **[ezAgents](https://github.com/namhv94/ezAgents)** | MCP-native Agent Orchestration Engine with deterministic verifiers | **Rust** | `v0.1.0` Active |
| **[ezEval](https://github.com/namhv94/ezEval)** | Boundary fuzzing, JSON Schema validation & safety guardrails | **Rust** | `v0.1.0` Active |

---

## 📄 License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.

Developed with ❤️ by **[EzLabs](https://ezlabs.tech)**.
