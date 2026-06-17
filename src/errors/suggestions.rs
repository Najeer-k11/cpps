use strsim::levenshtein;

/// Common C++ identifiers that might be typo targets
const COMMON_IDENTIFIERS: &[&str] = &[
    "printf", "println", "scanf", "cout", "cin", "endl", "cerr",
    "string", "vector", "map", "set", "list", "queue", "stack",
    "push_back", "emplace_back", "begin", "end", "size", "empty",
    "nullptr", "true", "false", "return", "include", "namespace",
    "std", "main", "int", "float", "double", "char", "bool", "void",
    "const", "static", "class", "struct", "public", "private", "protected",
    "virtual", "override", "template", "typename", "auto",
    "make_shared", "make_unique", "shared_ptr", "unique_ptr",
    "move", "forward", "swap",
];

/// Try to suggest a fix for an error message
pub fn suggest_fix(message: &str) -> Option<String> {
    // Look for "undeclared identifier" pattern
    if let Some(identifier) = extract_undeclared_identifier(message) {
        return find_similar(&identifier);
    }

    // Look for "did you mean" already in message (some compilers include this)
    if message.contains("did you mean") {
        return None; // Compiler already suggests
    }

    None
}

fn extract_undeclared_identifier(message: &str) -> Option<String> {
    // Pattern: "use of undeclared identifier 'xxx'"
    let re = regex::Regex::new(r"undeclared identifier '(\w+)'").ok()?;
    if let Some(caps) = re.captures(message) {
        return Some(caps[1].to_string());
    }

    // Pattern: "'xxx' was not declared in this scope"
    let re2 = regex::Regex::new(r"'(\w+)' was not declared").ok()?;
    if let Some(caps) = re2.captures(message) {
        return Some(caps[1].to_string());
    }

    None
}

fn find_similar(identifier: &str) -> Option<String> {
    let mut best_match: Option<(&str, usize)> = None;

    for &candidate in COMMON_IDENTIFIERS {
        let distance = levenshtein(identifier, candidate);
        if distance <= 2 && distance > 0 {
            if let Some((_, best_distance)) = best_match {
                if distance < best_distance {
                    best_match = Some((candidate, distance));
                }
            } else {
                best_match = Some((candidate, distance));
            }
        }
    }

    best_match.map(|(suggestion, _)| format!("Did you mean '{}'?", suggestion))
}
