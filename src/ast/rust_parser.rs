use quote::ToTokens;
use syn::{parse_file, ImplItem, Item};

pub struct RustSkeletonParser;

impl RustSkeletonParser {
    pub fn extract_skeleton(source_code: &str) -> Result<String, String> {
        let file =
            parse_file(source_code).map_err(|e| format!("Failed to parse Rust AST: {}", e))?;
        let mut skeleton_lines = Vec::new();

        for item in file.items {
            match item {
                Item::Struct(s) => {
                    skeleton_lines.push(s.to_token_stream().to_string());
                }
                Item::Enum(e) => {
                    skeleton_lines.push(e.to_token_stream().to_string());
                }
                Item::Trait(t) => {
                    skeleton_lines.push(t.to_token_stream().to_string());
                }
                Item::Type(t) => {
                    skeleton_lines.push(t.to_token_stream().to_string());
                }
                Item::Const(c) => {
                    skeleton_lines.push(c.to_token_stream().to_string());
                }
                Item::Fn(mut f) => {
                    // Replace function body with empty placeholder block
                    let dummy_block: syn::Block = syn::parse_str("{ /* ... */ }").unwrap();
                    *f.block = dummy_block;
                    skeleton_lines.push(f.to_token_stream().to_string());
                }
                Item::Impl(mut imp) => {
                    // Process items inside impl block
                    for impl_item in &mut imp.items {
                        if let ImplItem::Fn(f) = impl_item {
                            let dummy_block: syn::Block = syn::parse_str("{ /* ... */ }").unwrap();
                            f.block = dummy_block;
                        }
                    }
                    skeleton_lines.push(imp.to_token_stream().to_string());
                }
                Item::Use(u) => {
                    skeleton_lines.push(u.to_token_stream().to_string());
                }
                _ => {}
            }
        }

        // Format and clean tokens
        let joined = skeleton_lines.join("\n\n");
        Ok(clean_token_stream_formatting(&joined))
    }
}

fn clean_token_stream_formatting(tokens: &str) -> String {
    tokens
        .replace(" { ", " {\n    ")
        .replace(" ; ", ";\n")
        .replace(" } ", "\n}\n")
}
