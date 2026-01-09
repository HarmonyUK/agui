//! Basic Syntax Highlighting
//!
//! Provides simple regex-based syntax highlighting for common languages.
//! For production use, consider integrating tree-sitter for accurate parsing.

use std::collections::HashMap;

/// Token type for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// Regular text
    Text,
    /// Language keyword (fn, let, if, etc.)
    Keyword,
    /// String literal
    String,
    /// Numeric literal
    Number,
    /// Comment
    Comment,
    /// Function/method name
    Function,
    /// Type name
    Type,
    /// Operator (+, -, *, etc.)
    Operator,
    /// Punctuation (, ; : etc.)
    Punctuation,
    /// Variable name
    Variable,
    /// Attribute/decorator
    Attribute,
    /// Constant (all caps identifiers)
    Constant,
}

impl TokenType {
    /// Get the GPUI rgb color for this token type (VS Code dark theme)
    pub fn color(&self) -> u32 {
        match self {
            TokenType::Text => 0xcccccc,       // Default text
            TokenType::Keyword => 0x569cd6,   // Blue keywords
            TokenType::String => 0xce9178,    // Orange strings
            TokenType::Number => 0xb5cea8,    // Light green numbers
            TokenType::Comment => 0x6a9955,   // Green comments
            TokenType::Function => 0xdcdcaa,  // Yellow functions
            TokenType::Type => 0x4ec9b0,      // Teal types
            TokenType::Operator => 0xd4d4d4,  // Light gray operators
            TokenType::Punctuation => 0x808080, // Gray punctuation
            TokenType::Variable => 0x9cdcfe,  // Light blue variables
            TokenType::Attribute => 0xc586c0, // Purple attributes
            TokenType::Constant => 0x4fc1ff,  // Bright blue constants
        }
    }
}

/// A highlighted token span
#[derive(Debug, Clone)]
pub struct Token {
    /// Start position in the line
    pub start: usize,
    /// End position in the line
    pub end: usize,
    /// Token type
    pub token_type: TokenType,
}

/// Language definition for syntax highlighting
#[derive(Debug)]
pub struct LanguageDefinition {
    /// Keywords that should be highlighted
    pub keywords: Vec<&'static str>,
    /// Type keywords
    pub types: Vec<&'static str>,
    /// Built-in functions
    pub builtins: Vec<&'static str>,
    /// Single-line comment prefix
    pub line_comment: Option<&'static str>,
    /// Multi-line comment (start, end)
    pub block_comment: Option<(&'static str, &'static str)>,
    /// String delimiters
    pub string_delimiters: Vec<char>,
}

impl Default for LanguageDefinition {
    fn default() -> Self {
        Self {
            keywords: Vec::new(),
            types: Vec::new(),
            builtins: Vec::new(),
            line_comment: None,
            block_comment: None,
            string_delimiters: vec!['"', '\''],
        }
    }
}

/// Get language definition by language name
pub fn get_language_definition(language: &str) -> LanguageDefinition {
    match language.to_lowercase().as_str() {
        "rust" | "rs" => rust_definition(),
        "python" | "py" => python_definition(),
        "javascript" | "js" | "typescript" | "ts" => javascript_definition(),
        "go" | "golang" => go_definition(),
        "json" => json_definition(),
        "yaml" | "yml" => yaml_definition(),
        "toml" => toml_definition(),
        "bash" | "sh" | "shell" => bash_definition(),
        _ => LanguageDefinition::default(),
    }
}

fn rust_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec![
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
            "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
            "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
            "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
        ],
        types: vec![
            "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize",
            "str", "u8", "u16", "u32", "u64", "u128", "usize", "String", "Vec", "Option",
            "Result", "Box", "Rc", "Arc", "HashMap", "HashSet", "BTreeMap", "BTreeSet",
        ],
        builtins: vec![
            "println!", "print!", "format!", "vec!", "assert!", "assert_eq!", "debug!",
            "panic!", "todo!", "unimplemented!", "cfg!", "include!", "include_str!",
        ],
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
        string_delimiters: vec!['"'],
    }
}

fn python_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec![
            "False", "None", "True", "and", "as", "assert", "async", "await", "break",
            "class", "continue", "def", "del", "elif", "else", "except", "finally",
            "for", "from", "global", "if", "import", "in", "is", "lambda", "nonlocal",
            "not", "or", "pass", "raise", "return", "try", "while", "with", "yield",
        ],
        types: vec![
            "int", "float", "str", "bool", "list", "dict", "set", "tuple", "bytes",
            "type", "object", "Exception", "BaseException",
        ],
        builtins: vec![
            "print", "len", "range", "enumerate", "zip", "map", "filter", "sorted",
            "sum", "min", "max", "abs", "round", "open", "input", "type", "isinstance",
            "hasattr", "getattr", "setattr", "super", "property", "classmethod",
            "staticmethod", "iter", "next", "reversed", "slice", "format",
        ],
        line_comment: Some("#"),
        block_comment: None,
        string_delimiters: vec!['"', '\''],
    }
}

fn javascript_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec![
            "async", "await", "break", "case", "catch", "class", "const", "continue",
            "debugger", "default", "delete", "do", "else", "export", "extends", "false",
            "finally", "for", "function", "if", "import", "in", "instanceof", "let",
            "new", "null", "return", "static", "super", "switch", "this", "throw",
            "true", "try", "typeof", "undefined", "var", "void", "while", "with", "yield",
        ],
        types: vec![
            "Array", "Boolean", "Date", "Error", "Function", "Map", "Number", "Object",
            "Promise", "RegExp", "Set", "String", "Symbol", "WeakMap", "WeakSet",
            // TypeScript additions
            "any", "boolean", "number", "string", "void", "never", "unknown", "interface",
            "type", "enum", "namespace", "module", "declare", "readonly", "private",
            "protected", "public", "abstract", "implements",
        ],
        builtins: vec![
            "console", "window", "document", "fetch", "setTimeout", "setInterval",
            "clearTimeout", "clearInterval", "JSON", "Math", "parseInt", "parseFloat",
            "isNaN", "isFinite", "encodeURI", "decodeURI", "require", "module", "exports",
        ],
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
        string_delimiters: vec!['"', '\'', '`'],
    }
}

fn go_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec![
            "break", "case", "chan", "const", "continue", "default", "defer", "else",
            "fallthrough", "for", "func", "go", "goto", "if", "import", "interface",
            "map", "package", "range", "return", "select", "struct", "switch", "type",
            "var", "nil", "true", "false", "iota",
        ],
        types: vec![
            "bool", "byte", "complex64", "complex128", "error", "float32", "float64",
            "int", "int8", "int16", "int32", "int64", "rune", "string", "uint",
            "uint8", "uint16", "uint32", "uint64", "uintptr",
        ],
        builtins: vec![
            "append", "cap", "close", "complex", "copy", "delete", "imag", "len",
            "make", "new", "panic", "print", "println", "real", "recover",
        ],
        line_comment: Some("//"),
        block_comment: Some(("/*", "*/")),
        string_delimiters: vec!['"', '`'],
    }
}

fn json_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec!["true", "false", "null"],
        types: Vec::new(),
        builtins: Vec::new(),
        line_comment: None,
        block_comment: None,
        string_delimiters: vec!['"'],
    }
}

fn yaml_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec!["true", "false", "null", "yes", "no", "on", "off"],
        types: Vec::new(),
        builtins: Vec::new(),
        line_comment: Some("#"),
        block_comment: None,
        string_delimiters: vec!['"', '\''],
    }
}

fn toml_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec!["true", "false"],
        types: Vec::new(),
        builtins: Vec::new(),
        line_comment: Some("#"),
        block_comment: None,
        string_delimiters: vec!['"', '\''],
    }
}

fn bash_definition() -> LanguageDefinition {
    LanguageDefinition {
        keywords: vec![
            "if", "then", "else", "elif", "fi", "case", "esac", "for", "select",
            "while", "until", "do", "done", "in", "function", "time", "coproc",
            "return", "exit", "break", "continue", "true", "false",
        ],
        types: Vec::new(),
        builtins: vec![
            "echo", "printf", "read", "cd", "pwd", "export", "source", "alias",
            "unalias", "set", "unset", "shift", "test", "eval", "exec", "trap",
            "wait", "kill", "jobs", "bg", "fg", "disown", "suspend", "logout",
        ],
        line_comment: Some("#"),
        block_comment: None,
        string_delimiters: vec!['"', '\''],
    }
}

/// Syntax highlighter for a single language
#[derive(Debug)]
pub struct SyntaxHighlighter {
    definition: LanguageDefinition,
    keyword_set: HashMap<&'static str, TokenType>,
}

impl SyntaxHighlighter {
    /// Create a new highlighter for the given language
    pub fn new(language: &str) -> Self {
        let definition = get_language_definition(language);
        let mut keyword_set = HashMap::new();

        for kw in &definition.keywords {
            keyword_set.insert(*kw, TokenType::Keyword);
        }
        for ty in &definition.types {
            keyword_set.insert(*ty, TokenType::Type);
        }
        for bi in &definition.builtins {
            keyword_set.insert(*bi, TokenType::Function);
        }

        Self {
            definition,
            keyword_set,
        }
    }

    /// Highlight a single line of code
    pub fn highlight_line(&self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let start = i;

            // Check for line comment
            if let Some(comment_start) = self.definition.line_comment {
                if line[i..].starts_with(comment_start) {
                    tokens.push(Token {
                        start,
                        end: chars.len(),
                        token_type: TokenType::Comment,
                    });
                    break;
                }
            }

            // Check for string
            if self.definition.string_delimiters.contains(&chars[i]) {
                let delimiter = chars[i];
                i += 1;
                while i < chars.len() {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else if chars[i] == delimiter {
                        i += 1;
                        break;
                    } else {
                        i += 1;
                    }
                }
                tokens.push(Token {
                    start,
                    end: i,
                    token_type: TokenType::String,
                });
                continue;
            }

            // Check for number
            if chars[i].is_ascii_digit()
                || (chars[i] == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
            {
                while i < chars.len()
                    && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_')
                {
                    i += 1;
                }
                tokens.push(Token {
                    start,
                    end: i,
                    token_type: TokenType::Number,
                });
                continue;
            }

            // Check for identifier/keyword
            if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '!') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();

                let token_type = if let Some(&tt) = self.keyword_set.get(word.as_str()) {
                    tt
                } else if word.chars().all(|c| c.is_uppercase() || c == '_') && word.len() > 1 {
                    TokenType::Constant
                } else if i < chars.len() && chars[i] == '(' {
                    TokenType::Function
                } else if word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    TokenType::Type
                } else {
                    TokenType::Variable
                };

                tokens.push(Token {
                    start,
                    end: i,
                    token_type,
                });
                continue;
            }

            // Check for operator
            if "+-*/%=<>!&|^~?:".contains(chars[i]) {
                while i < chars.len() && "+-*/%=<>!&|^~?:".contains(chars[i]) {
                    i += 1;
                }
                tokens.push(Token {
                    start,
                    end: i,
                    token_type: TokenType::Operator,
                });
                continue;
            }

            // Check for punctuation
            if "()[]{}.,;@#".contains(chars[i]) {
                i += 1;
                tokens.push(Token {
                    start,
                    end: i,
                    token_type: TokenType::Punctuation,
                });
                continue;
            }

            // Skip whitespace or unknown characters
            i += 1;
        }

        tokens
    }

    /// Highlight multiple lines
    pub fn highlight(&self, content: &str) -> Vec<Vec<Token>> {
        content.lines().map(|line| self.highlight_line(line)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_keywords() {
        let highlighter = SyntaxHighlighter::new("rust");
        let tokens = highlighter.highlight_line("fn main() {");

        assert!(tokens.iter().any(|t| t.token_type == TokenType::Keyword));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Function));
    }

    #[test]
    fn test_string_highlighting() {
        let highlighter = SyntaxHighlighter::new("rust");
        let tokens = highlighter.highlight_line(r#"let s = "hello world";"#);

        assert!(tokens.iter().any(|t| t.token_type == TokenType::String));
    }

    #[test]
    fn test_comment_highlighting() {
        let highlighter = SyntaxHighlighter::new("rust");
        let tokens = highlighter.highlight_line("let x = 42; // comment");

        assert!(tokens.iter().any(|t| t.token_type == TokenType::Comment));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Number));
    }

    #[test]
    fn test_number_highlighting() {
        let highlighter = SyntaxHighlighter::new("rust");
        let tokens = highlighter.highlight_line("let x = 3.14;");

        let number_tokens: Vec<_> = tokens.iter().filter(|t| t.token_type == TokenType::Number).collect();
        assert!(!number_tokens.is_empty());
    }

    #[test]
    fn test_python() {
        let highlighter = SyntaxHighlighter::new("python");
        let tokens = highlighter.highlight_line("def hello(): # greeting");

        assert!(tokens.iter().any(|t| t.token_type == TokenType::Keyword));
        assert!(tokens.iter().any(|t| t.token_type == TokenType::Comment));
    }

    #[test]
    fn test_constant_detection() {
        let highlighter = SyntaxHighlighter::new("rust");
        let tokens = highlighter.highlight_line("const MAX_SIZE: usize = 100;");

        assert!(tokens.iter().any(|t| t.token_type == TokenType::Constant));
    }
}
