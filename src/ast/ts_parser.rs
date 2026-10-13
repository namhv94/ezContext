use regex::Regex;

pub struct TsSkeletonParser;

impl TsSkeletonParser {
    pub fn extract_skeleton(source_code: &str) -> Result<String, String> {
        let mut skeleton = Vec::new();
        let lines: Vec<&str> = source_code.lines().collect();
        let mut i = 0;

        let import_re =
            Regex::new(r"^(import\s+.*?from\s+['\x22].*?['\x22];?|import\s+['\x22].*?['\x22];?)")
                .unwrap();
        let type_re = Regex::new(r"^(export\s+)?(type|interface|enum)\s+([A-Za-z0-9_]+)").unwrap();
        let fn_start_re = Regex::new(r"^(export\s+)?(async\s+)?(function\s+[A-Za-z0-9_]+|\bconst\s+[A-Za-z0-9_]+\s*=\s*(async\s*)?\([^)]*\)\s*(=>|:))").unwrap();
        let class_re = Regex::new(r"^(export\s+)?(default\s+)?class\s+[A-Za-z0-9_]+").unwrap();

        while i < lines.len() {
            let line = lines[i].trim();

            if line.is_empty() {
                i += 1;
                continue;
            }

            // Keep imports
            if import_re.is_match(line) {
                skeleton.push(lines[i].to_string());
                i += 1;
                continue;
            }

            // Keep types, interfaces, enums
            if type_re.is_match(line) {
                let mut block = vec![lines[i]];
                if line.contains('{') && !line.contains('}') {
                    let mut depth = 1;
                    i += 1;
                    while i < lines.len() && depth > 0 {
                        block.push(lines[i]);
                        depth += lines[i].matches('{').count();
                        depth = depth.saturating_sub(lines[i].matches('}').count());
                        i += 1;
                    }
                    skeleton.push(block.join("\n"));
                    continue;
                } else {
                    skeleton.push(lines[i].to_string());
                    i += 1;
                    continue;
                }
            }

            // Classes
            if class_re.is_match(line) {
                skeleton.push(format!(
                    "{} {{ /* class members omitted in skeleton */ }}",
                    line.split('{').next().unwrap_or(line).trim()
                ));
                // Skip class body
                if line.contains('{') && !line.contains('}') {
                    let mut depth = 1;
                    i += 1;
                    while i < lines.len() && depth > 0 {
                        depth += lines[i].matches('{').count();
                        depth = depth.saturating_sub(lines[i].matches('}').count());
                        i += 1;
                    }
                    continue;
                }
                i += 1;
                continue;
            }

            // Function signatures
            if fn_start_re.is_match(line) {
                let sig = line.split('{').next().unwrap_or(line).trim();
                skeleton.push(format!("{} {{ /* ... */ }}", sig));
                // Skip body
                if line.contains('{') && !line.contains('}') {
                    let mut depth = 1;
                    i += 1;
                    while i < lines.len() && depth > 0 {
                        depth += lines[i].matches('{').count();
                        depth = depth.saturating_sub(lines[i].matches('}').count());
                        i += 1;
                    }
                    continue;
                }
                i += 1;
                continue;
            }

            i += 1;
        }

        Ok(skeleton.join("\n\n"))
    }
}
