pub mod py_parser;
pub mod rust_parser;
pub mod ts_parser;

use py_parser::PySkeletonParser;
use rust_parser::RustSkeletonParser;
use std::path::Path;
use ts_parser::TsSkeletonParser;

pub struct AstDispatcher;

impl AstDispatcher {
    pub fn extract_skeleton<P: AsRef<Path>>(path: P, content: &str) -> Result<String, String> {
        let p = path.as_ref();
        let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "rs" => RustSkeletonParser::extract_skeleton(content),
            "ts" | "tsx" | "js" | "jsx" => TsSkeletonParser::extract_skeleton(content),
            "py" => PySkeletonParser::extract_skeleton(content),
            _ => {
                // Fallback for non-AST languages: Return first 40 lines + summary
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() <= 60 {
                    Ok(content.to_string())
                } else {
                    let head = lines[..30].join("\n");
                    let tail = lines[lines.len() - 10..].join("\n");
                    Ok(format!(
                        "{}\n\n// ... [{} lines omitted by ezContext] ...\n\n{}",
                        head,
                        lines.len() - 40,
                        tail
                    ))
                }
            }
        }
    }

    /// Locates and returns the exact definition of a symbol within source code.
    pub fn read_symbol(content: &str, symbol_name: &str) -> Result<String, String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut target_start = None;

        for (idx, line) in lines.iter().enumerate() {
            if line.contains(symbol_name)
                && (line.contains("fn ")
                    || line.contains("def ")
                    || line.contains("struct ")
                    || line.contains("class ")
                    || line.contains("function ")
                    || line.contains("interface "))
            {
                target_start = Some(idx);
                break;
            }
        }

        let start =
            target_start.ok_or_else(|| format!("Symbol '{}' not found in source", symbol_name))?;
        // Slice up to 50 lines for the symbol definition
        let end = (start + 50).min(lines.len());
        Ok(lines[start..end].join("\n"))
    }
}
