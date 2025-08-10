use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Simple(SimpleCommand),
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
    pub name: String,
    pub args: Vec<String>,
    pub redirects: Vec<Redirect>,
    pub env_vars: HashMap<String, String>,
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
    pub items: Vec<String>,
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
    pub target: String,
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
} 