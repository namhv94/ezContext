use regex::Regex;

pub struct PySkeletonParser;

impl PySkeletonParser {
    pub fn extract_skeleton(source_code: &str) -> Result<String, String> {
        let mut skeleton = Vec::new();
        let lines: Vec<&str> = source_code.lines().collect();
        let mut i = 0;

        let import_re =
            Regex::new(r"^(import\s+[A-Za-z0-9_.]+|from\s+[A-Za-z0-9_.]+\s+import\s+.*)").unwrap();
        let class_re = Regex::new(r"^class\s+[A-Za-z0-9_]+(\(.*?\))?:").unwrap();
        let def_re =
            Regex::new(r"^(async\s+)?def\s+[A-Za-z0-9_]+\s*\(.*?\)(\s*->\s*[^:]+)?:").unwrap();
        let decorator_re = Regex::new(r"^@[A-Za-z0-9_.]+").unwrap();

        let mut pending_decorators = Vec::new();

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            // Keep decorators
            if decorator_re.is_match(trimmed) {
                pending_decorators.push(line);
                i += 1;
                continue;
            }

            // Keep imports
            if import_re.is_match(trimmed) {
                skeleton.push(line.to_string());
                i += 1;
                continue;
            }

            // Classes
            if class_re.is_match(trimmed) {
                for dec in pending_decorators.drain(..) {
                    skeleton.push(dec.to_string());
                }
                skeleton.push(format!("{}\n    pass", line));
                i += 1;
                continue;
            }

            // Functions / Methods
            if def_re.is_match(trimmed) {
                for dec in pending_decorators.drain(..) {
                    skeleton.push(dec.to_string());
                }
                // Check indentation
                let indent_len = line.len() - line.trim_start().len();
                let indent = " ".repeat(indent_len + 4);
                skeleton.push(format!("{}\n{}...", line, indent));

                // Skip function body until next statement with same or lower indentation
                let base_indent = indent_len;
                i += 1;
                while i < lines.len() {
                    let next_line = lines[i];
                    if next_line.trim().is_empty() {
                        i += 1;
                        continue;
                    }
                    let next_indent = next_line.len() - next_line.trim_start().len();
                    if next_indent <= base_indent {
                        break;
                    }
                    i += 1;
                }
                continue;
            }

            pending_decorators.clear();
            i += 1;
        }

        Ok(skeleton.join("\n\n"))
    }
}
