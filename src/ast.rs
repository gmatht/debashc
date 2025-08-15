use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Simple(SimpleCommand),
    BuiltinCommand(BuiltinCommand),
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
pub struct BuiltinCommand {
    pub name: String,
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
    ProcessSubstitutionInput(Box<Command>),  // <(command)
    ProcessSubstitutionOutput(Box<Command>), // >(command)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterExpansion {
    pub variable: String,
    pub operator: ParameterExpansionOperator,
}

impl std::fmt::Display for ParameterExpansion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.operator {
            ParameterExpansionOperator::UppercaseAll => write!(f, "${{{0}^^}}", self.variable),
            ParameterExpansionOperator::LowercaseAll => write!(f, "${{{0},,}}", self.variable),
            ParameterExpansionOperator::UppercaseFirst => write!(f, "${{{0}^}}", self.variable),
            ParameterExpansionOperator::RemoveLongestPrefix(pattern) => write!(f, "${{{0}##{1}}}", self.variable, pattern),
            ParameterExpansionOperator::RemoveShortestPrefix(pattern) => write!(f, "${{{0}#{1}}}", self.variable, pattern),
            ParameterExpansionOperator::RemoveLongestSuffix(pattern) => write!(f, "${{{0}%%{1}}}", self.variable, pattern),
            ParameterExpansionOperator::RemoveShortestSuffix(pattern) => write!(f, "${{{0}%{1}}}", self.variable, pattern),
            ParameterExpansionOperator::SubstituteAll(pattern, replacement) => write!(f, "${{{0}//{1}/{2}}}", self.variable, pattern, replacement),
            ParameterExpansionOperator::DefaultValue(default) => write!(f, "${{{0}:-{1}}}", self.variable, default),
            ParameterExpansionOperator::AssignDefault(default) => write!(f, "${{{0}:={1}}}", self.variable, default),
            ParameterExpansionOperator::ErrorIfUnset(error) => write!(f, "${{{0}:?{1}}}", self.variable, error),
            ParameterExpansionOperator::Basename => write!(f, "${{{0}##*/}}", self.variable),
            ParameterExpansionOperator::Dirname => write!(f, "${{{0}%/*}}", self.variable),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterExpansionOperator {
    // Case modification
    UppercaseAll,      // ^^
    LowercaseAll,      // ,,
    UppercaseFirst,    // ^
    
    // Substring removal
    RemoveLongestPrefix(String),  // ##pattern
    RemoveShortestPrefix(String), // #pattern
    RemoveLongestSuffix(String),  // %%pattern
    RemoveShortestSuffix(String), // %pattern
    
    // Pattern substitution
    SubstituteAll(String, String), // //pattern/replacement
    
    // Default values
    DefaultValue(String),          // :-default
    AssignDefault(String),         // :=default
    ErrorIfUnset(String),         // :?error
    
    // Path manipulation
    Basename,                      // ##*/
    Dirname,                       // %/*
}

// New AST nodes for expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Literal(String),
    Variable(String),
    ParameterExpansion(ParameterExpansion),
    Array(String, Vec<String>), // array_name, elements
    MapAccess(String, String), // map_name, key
    MapKeys(String), // !map[@] -> get keys of associative array
    MapLength(String), // #arr[@] -> get length of array
    Arithmetic(ArithmeticExpression),
    BraceExpansion(BraceExpansion),
    CommandSubstitution(Box<Command>),
    StringInterpolation(StringInterpolation),
}

impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Word::Literal(s) => write!(f, "{}", s),
            Word::Variable(var) => write!(f, "${}", var),
            Word::ParameterExpansion(pe) => {
                match &pe.operator {
                    ParameterExpansionOperator::UppercaseAll => write!(f, "${{{}}}", pe.variable),
                    ParameterExpansionOperator::LowercaseAll => write!(f, "${{{}}}", pe.variable),
                    ParameterExpansionOperator::UppercaseFirst => write!(f, "${{{}}}", pe.variable),
                    ParameterExpansionOperator::RemoveLongestPrefix(pattern) => write!(f, "${{{}}}##{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveShortestPrefix(pattern) => write!(f, "${{{}}}#{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveLongestSuffix(pattern) => write!(f, "${{{}}}%%{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveShortestSuffix(pattern) => write!(f, "${{{}}}%{}", pe.variable, pattern),
                    ParameterExpansionOperator::SubstituteAll(pattern, replacement) => write!(f, "${{{}}}//{}/{}", pe.variable, pattern, replacement),
                    ParameterExpansionOperator::DefaultValue(default) => write!(f, "${{{}}}:-{}", pe.variable, default),
                    ParameterExpansionOperator::AssignDefault(default) => write!(f, "${{{}}}:={}", pe.variable, default),
                    ParameterExpansionOperator::ErrorIfUnset(error) => write!(f, "${{{}}}:?{}", pe.variable, error),
                    ParameterExpansionOperator::Basename => write!(f, "${{{}}}##*/", pe.variable),
                    ParameterExpansionOperator::Dirname => write!(f, "${{{}}}%/*", pe.variable),
                }
            },
            Word::Array(name, elements) => write!(f, "{}=({})", name, elements.join(" ")),
            Word::MapAccess(map_name, key) => write!(f, "{}[{}]", map_name, key),
            Word::MapKeys(map_name) => write!(f, "!{}[@]", map_name),
            Word::MapLength(map_name) => write!(f, "#{}[@]", map_name),
            Word::Arithmetic(expr) => write!(f, "{}", expr.expression),
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
                write!(f, "{{{}}}", result)
            }
            Word::CommandSubstitution(_) => write!(f, "$(...)"),
            Word::StringInterpolation(interp) => {
                let mut result = String::new();
                for part in &interp.parts {
                    match part {
                        StringPart::Literal(s) => result.push_str(s),
                        StringPart::Variable(var) => result.push_str(&format!("${}", var)),
                        StringPart::ParameterExpansion(pe) => {
                            match &pe.operator {
                                ParameterExpansionOperator::UppercaseAll => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::LowercaseAll => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::UppercaseFirst => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::RemoveLongestPrefix(pattern) => result.push_str(&format!("${{{}}}##{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveShortestPrefix(pattern) => result.push_str(&format!("${{{}}}#{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveLongestSuffix(pattern) => result.push_str(&format!("${{{}}}%%{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveShortestSuffix(pattern) => result.push_str(&format!("${{{}}}%{}", pe.variable, pattern)),
                                ParameterExpansionOperator::SubstituteAll(pattern, replacement) => result.push_str(&format!("${{{}}}//{}/{}", pe.variable, pattern, replacement)),
                                ParameterExpansionOperator::DefaultValue(default) => result.push_str(&format!("${{{}}}:-{}", pe.variable, default)),
                                ParameterExpansionOperator::AssignDefault(default) => result.push_str(&format!("${{{}}}:={}", pe.variable, default)),
                                ParameterExpansionOperator::ErrorIfUnset(error) => result.push_str(&format!("${{{}}}:?{}", pe.variable, error)),
                                ParameterExpansionOperator::Basename => result.push_str(&format!("${{{}}}##*/", pe.variable)),
                                ParameterExpansionOperator::Dirname => result.push_str(&format!("${{{}}}%/*", pe.variable)),
                            }
                        }
                        StringPart::MapAccess(map_name, key) => result.push_str(&format!("{}[{}]", map_name, key)),
                        StringPart::MapKeys(map_name) => result.push_str(&format!("!{}[@]", map_name)),
                        StringPart::MapLength(map_name) => result.push_str(&format!("#{}[@]", map_name)),
                        StringPart::Arithmetic(expr) => result.push_str(&expr.expression),
                        StringPart::CommandSubstitution(_) => result.push_str("$(...)"),
                    }
                }
                write!(f, "{}", result)
            }
        }
    }
}

impl Word {
    /// Get a string representation of the word, suitable for display
    pub fn to_string(&self) -> String {
        match self {
            Word::Literal(s) => s.to_string(),
            Word::Variable(var) => format!("${}", var),
            Word::ParameterExpansion(pe) => {
                match &pe.operator {
                    ParameterExpansionOperator::UppercaseAll => format!("${{{}}}", pe.variable),
                    ParameterExpansionOperator::LowercaseAll => format!("${{{}}}", pe.variable),
                    ParameterExpansionOperator::UppercaseFirst => format!("${{{}}}", pe.variable),
                    ParameterExpansionOperator::RemoveLongestPrefix(pattern) => format!("${{{}}}##{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveShortestPrefix(pattern) => format!("${{{}}}#{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveLongestSuffix(pattern) => format!("${{{}}}%%{}", pe.variable, pattern),
                    ParameterExpansionOperator::RemoveShortestSuffix(pattern) => format!("${{{}}}%{}", pe.variable, pattern),
                    ParameterExpansionOperator::SubstituteAll(pattern, replacement) => format!("${{{}}}//{}/{}", pe.variable, pattern, replacement),
                    ParameterExpansionOperator::DefaultValue(default) => format!("${{{}}}:-{}", pe.variable, default),
                    ParameterExpansionOperator::AssignDefault(default) => format!("${{{}}}:={}", pe.variable, default),
                    ParameterExpansionOperator::ErrorIfUnset(error) => format!("${{{}}}:?{}", pe.variable, error),
                    ParameterExpansionOperator::Basename => format!("${{{}}}##*/", pe.variable),
                    ParameterExpansionOperator::Dirname => format!("${{{}}}%/*", pe.variable),
                }
            },
            Word::Array(name, elements) => format!("{}=({})", name, elements.join(" ")),
            Word::MapAccess(map_name, key) => format!("{}[{}]", map_name, key),
            Word::MapKeys(map_name) => format!("!{}[@]", map_name),
            Word::MapLength(map_name) => format!("#{}[@]", map_name),
            Word::Arithmetic(expr) => expr.expression.to_string(),
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
                        StringPart::ParameterExpansion(pe) => {
                            match &pe.operator {
                                ParameterExpansionOperator::UppercaseAll => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::LowercaseAll => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::UppercaseFirst => result.push_str(&format!("${{{}}}", pe.variable)),
                                ParameterExpansionOperator::RemoveLongestPrefix(pattern) => result.push_str(&format!("${{{}}}##{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveShortestPrefix(pattern) => result.push_str(&format!("${{{}}}#{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveLongestSuffix(pattern) => result.push_str(&format!("${{{}}}%%{}", pe.variable, pattern)),
                                ParameterExpansionOperator::RemoveShortestSuffix(pattern) => result.push_str(&format!("${{{}}}%{}", pe.variable, pattern)),
                                ParameterExpansionOperator::SubstituteAll(pattern, replacement) => result.push_str(&format!("${{{}}}//{}/{}", pe.variable, pattern, replacement)),
                                ParameterExpansionOperator::DefaultValue(default) => result.push_str(&format!("${{{}}}:-{}", pe.variable, default)),
                                ParameterExpansionOperator::AssignDefault(default) => result.push_str(&format!("${{{}}}:={}", pe.variable, default)),
                                ParameterExpansionOperator::ErrorIfUnset(error) => result.push_str(&format!("${{{}}}:?{}", pe.variable, error)),
                                ParameterExpansionOperator::Basename => result.push_str(&format!("${{{}}}##*/", pe.variable)),
                                ParameterExpansionOperator::Dirname => result.push_str(&format!("${{{}}}%/*", pe.variable)),
                            }
                        },
                        StringPart::MapAccess(map_name, key) => result.push_str(&format!("${{{}}}[{}]", map_name, key)),
                        StringPart::MapKeys(map_name) => result.push_str(&format!("${{!{}}}[@]", map_name)),
                        StringPart::MapLength(map_name) => result.push_str(&format!("${{#{}}}[@]", map_name)),
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
            Word::ParameterExpansion(pe) => pe.variable == other,
            Word::MapKeys(map_name) => map_name == other,
            Word::MapLength(map_name) => map_name == other,
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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub enum StringPart {
    Literal(String),
    Variable(String),
    ParameterExpansion(ParameterExpansion),
    MapAccess(String, String), // map_name, key
    MapKeys(String), // !map[@] -> get keys of associative array
    MapLength(String), // #arr[@] -> get length of array
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