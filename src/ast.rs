use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Simple(SimpleCommand),
    ShoptCommand(ShoptCommand),
    TestExpression(TestExpression),
    Pipeline(Pipeline),
    If(IfStatement),
    While(WhileLoop),
    For(ForLoop),
    Function(Function),
    Subshell(Box<Command>),
    Background(Box<Command>),
    Block(Block),
    BlankLine,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleCommand {
    pub name: Word,
    pub args: Vec<Word>,
    pub redirects: Vec<Redirect>,
    pub env_vars: HashMap<String, Word>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShoptCommand {
    pub option: String,
    pub enable: bool, // true for -s (set), false for -u (unset)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline {
    pub commands: Vec<Command>,
    pub operators: Vec<PipeOperator>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PipeOperator {
    Pipe,      // |
    And,       // &&
    Or,        // ||
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStatement {
    pub condition: Box<Command>,
    pub then_branch: Box<Command>,
    pub else_branch: Option<Box<Command>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileLoop {
    pub condition: Box<Command>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForLoop {
    pub variable: String,
    pub items: Vec<Word>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub commands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Redirect {
    pub fd: Option<i32>,
    pub operator: RedirectOperator,
    pub target: Word,
    pub heredoc_body: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectOperator {
    Input,     // <
    Output,    // >
    Append,    // >>
    InputOutput, // <>
    Heredoc,   // <<
    HeredocTabs, // <<-
    HereString, // <<<
}

// New AST nodes for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Literal(String),
    Variable(String),
    MapAccess(String, String), // map_name, key
    Arithmetic(ArithmeticExpression),
    BraceExpansion(BraceExpansion),
    CommandSubstitution(Box<Command>),
    StringInterpolation(StringInterpolation),
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Word {
    /// Get a string representation of the word, suitable for display
    pub fn to_string(&self) -> String {
        match self {
            Word::Literal(s) => s.clone(),
            Word::Variable(var) => format!("${}", var),
            Word::MapAccess(map_name, key) => format!("{}[{}]", map_name, key),
            Word::Arithmetic(expr) => expr.expression.clone(),
            Word::BraceExpansion(expansion) => {
                let mut result = String::new();
                if let Some(ref prefix) = expansion.prefix {
                    result.push_str(prefix);
                }
                for (i, item) in expansion.items.iter().enumerate() {
                    if i > 0 {
                        result.push(',');
                    }
                    match item {
                        BraceItem::Literal(s) => result.push_str(s),
                        BraceItem::Range(range) => {
                            result.push_str(&range.start);
                            result.push_str("..");
                            result.push_str(&range.end);
                            if let Some(ref step) = range.step {
                                result.push_str("..");
                                result.push_str(step);
                            }
                        }
                        BraceItem::Sequence(seq) => {
                            result.push_str(&seq.join(","));
                        }
                    }
                }
                if let Some(ref suffix) = expansion.suffix {
                    result.push_str(suffix);
                }
                format!("{{{}}}", result)
            }
            Word::CommandSubstitution(_) => "$(...)".to_string(),
            Word::StringInterpolation(interp) => {
                let mut result = String::new();
                for part in &interp.parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Variable(var) => result.push_str(&format!("${}", var)),
                        StringPart::MapAccess(map_name, key) => result.push_str(&format!("${{{}}}[{}]", map_name, key)),
                        StringPart::Arithmetic(expr) => result.push_str(&expr.expression),
                        StringPart::CommandSubstitution(_) => result.push_str("$(...)"),
                    }
                }
                format!("\"{}\"", result)
            }
        }
    }

    /// Get the raw string value if this is a literal, or convert to string otherwise
    pub fn as_str(&self) -> &str {
        match self {
            Word::Literal(s) => s,
            _ => "",
        }
    }

    /// Check if this word is a literal with the given value
    pub fn is_literal(&self, value: &str) -> bool {
        matches!(self, Word::Literal(s) if s == value)
    }

    /// Extract the variable name if this is a variable
    pub fn as_variable(&self) -> Option<&str> {
        match self {
            Word::Variable(var) => Some(var),
            Word::MapAccess(map_name, _) => Some(map_name),
            _ => None,
        }
    }

    /// Check if this word contains a specific character
    pub fn contains(&self, ch: char) -> bool {
        match self {
            Word::Literal(s) => s.contains(ch),
            Word::Variable(var) => var.contains(ch),
            Word::MapAccess(map_name, key) => map_name.contains(ch) || key.contains(ch),
            Word::Arithmetic(expr) => expr.expression.contains(ch),
            Word::BraceExpansion(expansion) => {
                if let Some(ref prefix) = expansion.prefix {
                    if prefix.contains(ch) { return true; }
                }
                if let Some(ref suffix) = expansion.suffix {
                    if suffix.contains(ch) { return true; }
                }
                for item in &expansion.items {
                    match item {
                        BraceItem::Literal(s) => if s.contains(ch) { return true; },
                        BraceItem::Range(range) => {
                            if range.start.contains(ch) || range.end.contains(ch) { return true; }
                            if let Some(ref step) = range.step {
                                if step.contains(ch) { return true; }
                            }
                        }
                        BraceItem::Sequence(seq) => {
                            for s in seq {
                                if s.contains(ch) { return true; }
                            }
                        }
                    }
                }
                false
            }
            Word::CommandSubstitution(_) => false,
            Word::StringInterpolation(interp) => {
                for part in &interp.parts {
                    match part {
                        StringPart::Literal(s) => { if s.contains(ch) { return true; } }
                        StringPart::Variable(var) => { if var.contains(ch) { return true; } }
                        StringPart::MapAccess(map_name, key) => { if map_name.contains(ch) || key.contains(ch) { return true; } }
                        StringPart::Arithmetic(expr) => { if expr.expression.contains(ch) { return true; } }
                        StringPart::CommandSubstitution(_) => { return false; }
                    }
                }
                false
            }
        }
    }

    /// Split the word by a delimiter
    pub fn splitn(&self, n: usize, pat: char) -> Vec<String> {
        match self {
            Word::Literal(s) => s.splitn(n, pat).map(|s| s.to_string()).collect(),
            Word::Variable(var) => var.splitn(n, pat).map(|s| s.to_string()).collect(),
            Word::MapAccess(map_name, key) => {
                let mut result = map_name.splitn(n, pat).map(|s| s.to_string()).collect::<Vec<_>>();
                if result.len() < n {
                    result.extend(key.splitn(n - result.len(), pat).map(|s| s.to_string()));
                }
                result
            }
            Word::Arithmetic(expr) => expr.expression.splitn(n, pat).map(|s| s.to_string()).collect(),
            _ => vec![self.to_string()],
        }
    }

    /// Strip a prefix from the word
    pub fn strip_prefix(&self, prefix: &str) -> Option<String> {
        match self {
            Word::Literal(s) => s.strip_prefix(prefix).map(|s| s.to_string()),
            Word::Variable(var) => var.strip_prefix(prefix).map(|s| s.to_string()),
            Word::MapAccess(map_name, key) => {
                if let Some(stripped) = map_name.strip_prefix(prefix) {
                    Some(format!("{}[{}]", stripped, key))
                } else {
                    None
                }
            }
            Word::Arithmetic(expr) => expr.expression.strip_prefix(prefix).map(|s| s.to_string()),
            _ => None,
        }
    }

    /// Strip a prefix from the word (char version)
    pub fn strip_prefix_char(&self, prefix: char) -> Option<String> {
        match self {
            Word::Literal(s) => s.strip_prefix(prefix).map(|s| s.to_string()),
            Word::Variable(var) => var.strip_prefix(prefix).map(|s| s.to_string()),
            Word::MapAccess(map_name, key) => {
                if let Some(stripped) = map_name.strip_prefix(prefix) {
                    Some(format!("{}[{}]", stripped, key))
                } else {
                    None
                }
            }
            Word::Arithmetic(expr) => expr.expression.strip_prefix(prefix).map(|s| s.to_string()),
            _ => None,
        }
    }

    /// Replace occurrences of a pattern in the word
    pub fn replace(&self, from: &str, to: &str) -> String {
        match self {
            Word::Literal(s) => s.replace(from, to),
            Word::Variable(var) => var.replace(from, to),
            Word::MapAccess(map_name, key) => {
                let new_map_name = map_name.replace(from, to);
                let new_key = key.replace(from, to);
                format!("{}[{}]", new_map_name, new_key)
            }
            Word::Arithmetic(expr) => expr.expression.replace(from, to),
            _ => self.to_string(),
        }
    }

    /// Replace occurrences of a character in the word
    pub fn replace_char(&self, from: char, to: &str) -> String {
        match self {
            Word::Literal(s) => s.replace(from, to),
            Word::Variable(var) => var.replace(from, to),
            Word::MapAccess(map_name, key) => {
                let new_map_name = map_name.replace(from, to);
                let new_key = key.replace(from, to);
                format!("{}[{}]", new_map_name, new_key)
            }
            Word::Arithmetic(expr) => expr.expression.replace(from, to),
            _ => self.to_string(),
        }
    }
}

impl std::ops::Deref for Word {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        match self {
            Word::Literal(s) => s,
            _ => "",
        }
    }
}

impl PartialEq<str> for Word {
    fn eq(&self, other: &str) -> bool {
        match self {
            Word::Literal(s) => s == other,
            Word::Variable(var) => var == other,
            Word::Arithmetic(expr) => expr.expression == other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for Word {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<String> for Word {
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

/// Helper trait for converting Vec<Word> to Vec<String>
pub trait WordVecExt {
    fn to_strings(&self) -> Vec<String>;
    fn join(&self, separator: &str) -> String;
}

impl WordVecExt for Vec<Word> {
    fn to_strings(&self) -> Vec<String> {
        self.iter().map(|w| w.to_string()).collect()
    }
    
    fn join(&self, separator: &str) -> String {
        self.iter().map(|w| w.to_string()).collect::<Vec<_>>().join(separator)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArithmeticExpression {
    pub expression: String,
    pub tokens: Vec<ArithmeticToken>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticToken {
    Number(String),
    Variable(String),
    Operator(String),
    ParenOpen,
    ParenClose,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BraceExpansion {
    pub prefix: Option<String>,
    pub items: Vec<BraceItem>,
    pub suffix: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BraceItem {
    Literal(String),
    Range(BraceRange),
    Sequence(Vec<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BraceRange {
    pub start: String,
    pub end: String,
    pub step: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringInterpolation {
    pub parts: Vec<StringPart>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String),
    Variable(String),
    MapAccess(String, String), // map_name, key
    Arithmetic(ArithmeticExpression),
    CommandSubstitution(Box<Command>),
} 

#[derive(Debug, Clone, PartialEq)]
pub struct TestExpression {
    pub expression: String,
    pub modifiers: TestModifiers,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestModifiers {
    pub extglob: bool,
    pub nocasematch: bool,
    pub globstar: bool,
    pub nullglob: bool,
    pub failglob: bool,
    pub dotglob: bool,
}

impl Default for TestModifiers {
    fn default() -> Self {
        Self {
            extglob: false,
            nocasematch: false,
            globstar: false,
            nullglob: false,
            failglob: false,
            dotglob: false,
        }
    }
} 