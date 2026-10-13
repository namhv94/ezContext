use ezcontext::ast::AstDispatcher;
use ezcontext::mcp::protocol::JsonRpcRequest;
use ezcontext::mcp::McpStdioServer;
use ezcontext::token::TokenEstimator;
use serde_json::json;

#[test]
fn test_rust_skeleton_extraction() {
    let code = r#"
        pub struct User {
            pub id: u64,
            pub name: String,
        }

        impl User {
            pub fn new(id: u64, name: &str) -> Self {
                let formatted = format!("User: {}", name);
                println!("Created {}", formatted);
                Self { id, name: formatted }
            }

            pub fn get_name(&self) -> &str {
                &self.name
            }
        }
    "#;

    let skeleton = AstDispatcher::extract_skeleton("src/user.rs", code).expect("Failed to parse");
    assert!(skeleton.contains("pub struct User"));
    assert!(skeleton.contains("pub fn new"));
    assert!(skeleton.contains("pub fn get_name"));
    // Internal variable should be stripped from skeleton
    assert!(!skeleton.contains("let formatted = format!"));
}

#[test]
fn test_typescript_skeleton_extraction() {
    let code = r#"
        import { useState } from 'react';

        export interface UserProfile {
            id: string;
            age: number;
        }

        export async function fetchProfile(userId: string): Promise<UserProfile> {
            const resp = await fetch(`/api/users/${userId}`);
            const data = await resp.json();
            console.log("Fetched user", data);
            return data;
        }
    "#;

    let skeleton = AstDispatcher::extract_skeleton("app.ts", code).expect("Failed to parse");
    assert!(skeleton.contains("export interface UserProfile"));
    assert!(skeleton.contains("export async function fetchProfile"));
    // Inner fetch should be stripped
    assert!(!skeleton.contains("const resp = await fetch"));
}

#[test]
fn test_python_skeleton_extraction() {
    let code = r#"
        import os
        import sys

        class PaymentHandler:
            def __init__(self, api_key: str):
                self.api_key = api_key
                self.is_connected = True

            def process_charge(self, amount: int) -> bool:
                url = "https://api.stripe.com/charge"
                # Long inner logic
                return True
    "#;

    let skeleton = AstDispatcher::extract_skeleton("payment.py", code).expect("Failed to parse");
    assert!(skeleton.contains("class PaymentHandler:"));
    assert!(skeleton.contains("def __init__"));
    assert!(skeleton.contains("def process_charge"));
    // Inner url should be replaced by ...
    assert!(!skeleton.contains("https://api.stripe.com/charge"));
}

#[test]
fn test_token_savings() {
    let original = "pub fn heavy() {\n".to_string()
        + &"    println!(\"heavy computation\");\n".repeat(50)
        + "}";
    let skeleton = "pub fn heavy() { /* ... */ }";

    let savings = TokenEstimator::calculate_savings(&original, skeleton);
    assert!(savings.tokens_saved > 0);
    assert!(savings.percentage_saved > 60.0);
}

#[test]
fn test_mcp_protocol_messages() {
    let init_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "initialize".to_string(),
        params: json!({}),
    };
    let init_res = McpStdioServer::handle_request(&init_req);
    assert!(init_res.result.is_some());
    assert_eq!(
        init_res.result.unwrap()["serverInfo"]["name"],
        "ezcontext-mcp-rust"
    );

    let list_req = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/list".to_string(),
        params: json!({}),
    };
    let list_res = McpStdioServer::handle_request(&list_req);
    let tools = list_res.result.unwrap()["tools"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(tools.len(), 4);
}
