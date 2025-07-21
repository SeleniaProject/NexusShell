use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<AstNode>),           // 1
    Sequence(Vec<AstNode>),          // 2
    Pipeline(Vec<AstNode>),          // 3
    Command(Command),                // 4
    Argument(Argument),              // 5
    Assignment(String, Box<AstNode>),// 6
    Variable(String),                // 7
    Word(String),                    // 8
    StringLit(String),               // 9
    NumberLit(i64),                  // 10
    UnaryOp {                        // 11
        op: UnaryOperator,
        expr: Box<AstNode>,
    },
    BinaryOp {                       // 12
        op: BinaryOperator,
        left: Box<AstNode>,
        right: Box<AstNode>,
    },
    TernaryOp {                      // 13
        cond: Box<AstNode>,
        then_br: Box<AstNode>,
        else_br: Box<AstNode>,
    },
    If {                             // 14
        cond: Box<AstNode>,
        then_br: Box<AstNode>,
        else_br: Option<Box<AstNode>>,
    },
    While {                          // 15
        cond: Box<AstNode>,
        body: Box<AstNode>,
    },
    For {                            // 16
        var: String,
        iter: Box<AstNode>,
        body: Box<AstNode>,
    },
    Case {                           // 17
        expr: Box<AstNode>,
        arms: Vec<(AstNode, AstNode)>,
    },
    FunctionDecl {                   // 18
        name: String,
        params: Vec<String>,
        body: Box<AstNode>,
    },
    FunctionCall {                   // 19
        name: String,
        args: Vec<AstNode>,
    },
    Subshell(Box<AstNode>),          // 20
    Group(Box<AstNode>),             // 21
    RedirectIn {                    // 22
        file: String,
    },
    RedirectOut {                   // 23
        file: String,
        append: bool,
    },
    Heredoc {                       // 24
        delimiter: String,
        body: String,
    },
    Background(Box<AstNode>),        // 25
    And(Box<AstNode>, Box<AstNode>), // 26
    Or(Box<AstNode>, Box<AstNode>),  // 27
    Return(Option<Box<AstNode>>),    // 28
    Break,                           // 29
    Continue,                        // 30
    Array(Vec<AstNode>),             // 31
    Index {                          // 32
        array: Box<AstNode>,
        index: Box<AstNode>,
    },
    Match {                          // 33
        expr: Box<AstNode>,
        arms: Vec<(AstNode, AstNode)>,
    },
    Switch {                         // 34
        expr: Box<AstNode>,
        arms: Vec<(AstNode, AstNode)>,
    },
    NoOp,                            // 35
}

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub args: Vec<Argument>,
    pub redirects: Vec<AstNode>, // RedirectIn / RedirectOut / Heredoc variants
}

#[derive(Debug, Clone)]
pub enum Argument {
    Word(String),
    String(String),
    Number(i64),
    Variable(String),
    Array(Vec<Argument>),
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Ne,
    Gt,
    Lt,
    And,
    Or,
}

impl Default for AstNode {
    fn default() -> Self {
        AstNode::NoOp
    }
} 