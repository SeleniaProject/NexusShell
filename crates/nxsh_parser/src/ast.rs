//! Abstract Syntax Tree for NexusShell
//!
//! This module provides a comprehensive AST representation for all shell
//! constructs with zero-copy string references and type-safe operations.

use std::fmt;
use std::path::Path;

/// Standalone Command structure for executor compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct Command<'src> {
    pub name: Box<AstNode<'src>>,
    pub args: Vec<Argument<'src>>,
    pub redirections: Vec<Redirection<'src>>,
    pub background: bool,
}

/// Standalone Pipeline structure for executor compatibility
#[derive(Debug, Clone, PartialEq)]
pub struct Pipeline<'src> {
    pub elements: Vec<AstNode<'src>>,
    pub operators: Vec<PipeOperator>,
}

/// Command arguments with various types
#[derive(Debug, Clone, PartialEq)]
pub enum Argument<'src> {
    Word(&'src str),
    String(&'src str),
    Number(i64),
    Variable(&'src str),
    CommandSubstitution(&'src str),
    ProcessSubstitution(&'src str),
    Glob(GlobPattern<'src>),
    BraceExpansion(Vec<&'src str>),
    TildeExpansion(&'src str),
    ArithmeticExpansion(&'src str),
}

/// Redirection types for I/O operations
#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionType {
    Input,       // <
    Output,      // >
    Append,      // >>
    Error,       // 2>
    ErrorAppend, // 2>>
    Both,        // &>
    BothAppend,  // &>>
    Heredoc,     // <<
    Herestring,  // <<<
    InputOutput, // <>
    Pipe,        // |
    ErrorPipe,   // |&
}

/// Main AST node type with zero-copy string references
#[derive(Debug, Clone, PartialEq, Default)]
pub enum AstNode<'src> {
    // Program structure
    Program(Vec<AstNode<'src>>),
    StatementList(Vec<AstNode<'src>>),

    // Pipelines and commands
    Pipeline {
        elements: Vec<AstNode<'src>>,
        operators: Vec<PipeOperator>,
    },
    Command {
        name: Box<AstNode<'src>>,
        args: Vec<AstNode<'src>>,
        redirections: Vec<Redirection<'src>>,
        background: bool,
    },
    SimpleCommand {
        name: &'src str,
        args: Vec<&'src str>,
    },
    CompoundCommand(Box<AstNode<'src>>),

    // Control flow
    If {
        condition: Box<AstNode<'src>>,
        then_branch: Box<AstNode<'src>>,
        elif_branches: Vec<(AstNode<'src>, AstNode<'src>)>,
        else_branch: Option<Box<AstNode<'src>>>,
    },
    For {
        variable: &'src str,
        iterable: Box<AstNode<'src>>,
        body: Box<AstNode<'src>>,
        is_async: bool,
    },
    ForC {
        init: Option<Box<AstNode<'src>>>,
        condition: Option<Box<AstNode<'src>>>,
        update: Option<Box<AstNode<'src>>>,
        body: Box<AstNode<'src>>,
    },
    While {
        condition: Box<AstNode<'src>>,
        body: Box<AstNode<'src>>,
    },
    Until {
        condition: Box<AstNode<'src>>,
        body: Box<AstNode<'src>>,
    },
    Case {
        expr: Box<AstNode<'src>>,
        arms: Vec<CaseArm<'src>>,
    },
    Select {
        variable: &'src str,
        options: Option<Box<AstNode<'src>>>,
        body: Box<AstNode<'src>>,
    },

    // Modern control structures with enhanced pattern matching
    Match {
        expr: Box<AstNode<'src>>,
        arms: Vec<MatchArm<'src>>,
        is_exhaustive: bool,
    },
    MatchExpression {
        expr: Box<AstNode<'src>>,
        arms: Vec<MatchArm<'src>>,
        default_arm: Option<Box<AstNode<'src>>>,
    },

    // Destructuring assignment
    DestructureAssignment {
        pattern: Pattern<'src>,
        value: Box<AstNode<'src>>,
        is_local: bool,
    },

    // Pattern-based variable binding
    LetBinding {
        pattern: Pattern<'src>,
        value: Box<AstNode<'src>>,
        is_mutable: bool,
    },

    Try {
        body: Box<AstNode<'src>>,
        catch_clauses: Vec<CatchClause<'src>>,
        finally_clause: Option<Box<AstNode<'src>>>,
    },

    // Functions
    Function {
        name: &'src str,
        params: Vec<Parameter<'src>>,
        body: Box<AstNode<'src>>,
        is_async: bool,
        generics: Vec<&'src str>,
    },
    FunctionDeclaration {
        name: &'src str,
        params: Vec<Parameter<'src>>,
        body: Box<AstNode<'src>>,
        is_async: bool,
        generics: Vec<&'src str>,
    },
    FunctionCall {
        name: Box<AstNode<'src>>,
        args: Vec<AstNode<'src>>,
        is_async: bool,
        generics: Vec<&'src str>,
    },
    /// Macro declaration (compile-time like expansion)
    MacroDeclaration {
        name: &'src str,
        params: Vec<&'src str>,
        body: Box<AstNode<'src>>, // Typically a StatementList / BraceGroup
    },
    /// Macro invocation node
    MacroInvocation {
        name: &'src str,
        args: Vec<AstNode<'src>>,
    },
    // Anonymous closure (lambda)
    Closure {
        params: Vec<Parameter<'src>>,
        body: Box<AstNode<'src>>,
        captures: Vec<&'src str>,
        is_async: bool,
    },

    // Variable operations
    Assignment {
        name: &'src str,
        operator: AssignmentOperator,
        value: Box<AstNode<'src>>,
        is_local: bool,
        is_export: bool,
        is_readonly: bool,
    },
    VariableAssignment {
        name: &'src str,
        operator: AssignmentOperator,
        value: Box<AstNode<'src>>,
        is_local: bool,
        is_export: bool,
        is_readonly: bool,
    },
    ArrayAssignment {
        name: &'src str,
        elements: Vec<ArrayElement<'src>>,
        is_local: bool,
        is_export: bool,
    },

    // Expressions
    BinaryExpression {
        left: Box<AstNode<'src>>,
        operator: BinaryOperator,
        right: Box<AstNode<'src>>,
    },
    UnaryExpression {
        operator: UnaryOperator,
        operand: Box<AstNode<'src>>,
    },
    PostfixExpression {
        operand: Box<AstNode<'src>>,
        operator: PostfixOperator,
    },
    ConditionalExpression {
        condition: Box<AstNode<'src>>,
        then_expr: Box<AstNode<'src>>,
        else_expr: Box<AstNode<'src>>,
    },

    // Test expressions
    TestExpression {
        condition: Box<AstNode<'src>>,
        is_extended: bool, // [[ ]] vs [ ]
    },
    TestBinary {
        left: Box<AstNode<'src>>,
        operator: TestOperator,
        right: Box<AstNode<'src>>,
    },
    TestUnary {
        operator: TestUnaryOperator,
        operand: Box<AstNode<'src>>,
    },

    // Expansions
    VariableExpansion {
        name: &'src str,
        modifier: Option<ParameterModifier<'src>>,
    },
    CommandSubstitution {
        command: Box<AstNode<'src>>,
        is_legacy: bool, // backticks vs $()
    },
    ArithmeticExpansion {
        expr: Box<AstNode<'src>>,
        is_legacy: bool, // $[] vs $(())
    },
    ProcessSubstitution {
        command: Box<AstNode<'src>>,
        direction: ProcessSubstitutionDirection,
    },
    PathnameExpansion {
        pattern: GlobPattern<'src>,
    },
    BraceExpansion {
        elements: Vec<BraceElement<'src>>,
    },
    TildeExpansion {
        user: Option<&'src str>,
    },

    // Literals
    Word(&'src str),
    StringLiteral {
        value: &'src str,
        quote_type: QuoteType,
    },
    NumberLiteral {
        value: &'src str,
        number_type: NumberType,
    },
    Array(Vec<AstNode<'src>>),

    // Grouping
    Subshell(Box<AstNode<'src>>),
    BraceGroup(Box<AstNode<'src>>),

    // Control flow statements
    Background(Box<AstNode<'src>>),
    Return(Option<Box<AstNode<'src>>>),
    Break(Option<&'src str>),    // Optional label
    Continue(Option<&'src str>), // Optional label
    Exit(Option<Box<AstNode<'src>>>),

    // Logical operators for command sequences
    LogicalAnd {
        left: Box<AstNode<'src>>,
        right: Box<AstNode<'src>>,
    },
    LogicalOr {
        left: Box<AstNode<'src>>,
        right: Box<AstNode<'src>>,
    },
    Sequence {
        left: Box<AstNode<'src>>,
        right: Box<AstNode<'src>>,
    },

    // Argument collections
    ArgumentList(Vec<AstNode<'src>>),
    Variable(&'src str),

    // Modern language features
    ImportStatement {
        module_path: ModulePath<'src>,
        import_type: ImportType<'src>,
    },
    ModuleDeclaration {
        name: &'src str,
        body: Box<AstNode<'src>>,
        visibility: Visibility,
    },
    TypeDeclaration {
        name: &'src str,
        type_def: TypeDefinition<'src>,
    },

    // Async/await
    AsyncBlock(Box<AstNode<'src>>),
    AwaitExpression(Box<AstNode<'src>>),
    YieldExpression(Option<Box<AstNode<'src>>>),

    // Error handling
    ThrowStatement(Box<AstNode<'src>>),

    // Comments and metadata
    Comment(&'src str),

    // Special nodes
    #[default]
    Empty,
    Error {
        message: String,
        location: SourceLocation,
    },
}

/// Pipeline operators
#[derive(Debug, Clone, PartialEq)]
pub enum PipeOperator {
    Pipe,               // |
    LogicalOr,          // ||
    LogicalAnd,         // &&
    ObjectPipe,         // |>
    ObjectPipeParallel, // ||>
    Background,         // &
    Semicolon,          // ;
}

/// Redirection types
#[derive(Debug, Clone, PartialEq)]
pub struct Redirection<'src> {
    pub fd: Option<u32>,
    pub operator: RedirectionOperator,
    pub target: RedirectionTarget<'src>,
    pub redir_type: RedirectionType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionOperator {
    Output,           // >
    OutputAppend,     // >>
    Input,            // <
    InputOutput,      // <>
    OutputBoth,       // &>
    OutputBothAppend, // &>>
    HereDocument,     // <<
    HereString,       // <<<
    DuplicateInput,   // <&
    DuplicateOutput,  // >&
}

#[derive(Debug, Clone, PartialEq)]
pub enum RedirectionTarget<'src> {
    File(Box<AstNode<'src>>),
    FileDescriptor(u32),
    Close, // &-
    HereDoc {
        delimiter: &'src str,
        content: &'src str,
        expand: bool,
    },
}

impl<'src> AsRef<Path> for RedirectionTarget<'src> {
    fn as_ref(&self) -> &Path {
        match self {
            RedirectionTarget::File(node) => {
                // Extract file path from the AST node
                match **node {
                    AstNode::Word(path) => Path::new(path),
                    AstNode::StringLiteral { value: path, .. } => Path::new(path),
                    AstNode::Variable(_var) => {
                        // For variables, we need runtime evaluation
                        // Use a more descriptive temporary path
                        Path::new("/tmp/nexus_redirect_var")
                    }
                    AstNode::CommandSubstitution { .. } => {
                        // Command substitution results need runtime evaluation
                        // Use a more descriptive temporary path
                        Path::new("/tmp/nexus_redirect_cmd")
                    }
                    _ => {
                        // Fallback for other node types - use descriptive path
                        Path::new("/tmp/nexus_redirect_expr")
                    }
                }
            }
            RedirectionTarget::FileDescriptor(_fd) => Path::new("/dev/null"),
            RedirectionTarget::Close => Path::new("/dev/null"),
            RedirectionTarget::HereDoc { .. } => Path::new("/tmp/heredoc"),
        }
    }
}

/// Assignment operators
#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOperator {
    Assign,    // =
    AddAssign, // +=
    SubAssign, // -=
    MulAssign, // *=
    DivAssign, // /=
    ModAssign, // %=
    Append,    // >>=
    Prepend,   // <<=
}

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,      // +
    Subtract, // -
    Multiply, // *
    Divide,   // /
    Modulo,   // %
    Power,    // **

    // Comparison
    Equal,        // ==
    NotEqual,     // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    Match,        // =~
    NotMatch,     // !~

    // Logical
    LogicalAnd, // &&
    LogicalOr,  // ||

    // Bitwise
    BitwiseAnd, // &
    BitwiseOr,  // |
    BitwiseXor, // ^
    LeftShift,  // <<
    RightShift, // >>
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,       // +
    Minus,      // -
    LogicalNot, // !
    BitwiseNot, // ~
}

/// Postfix operators
#[derive(Debug, Clone, PartialEq)]
pub enum PostfixOperator {
    Increment, // ++
    Decrement, // --
}

/// Test operators for [ ] and [[ ]]
#[derive(Debug, Clone, PartialEq)]
pub enum TestOperator {
    // String comparison
    StringEqual,    // =
    StringNotEqual, // !=
    StringLess,     // <
    StringGreater,  // >
    StringMatch,    // =~
    StringNotMatch, // !~

    // Numeric comparison
    NumericEqual,        // -eq
    NumericNotEqual,     // -ne
    NumericLess,         // -lt
    NumericLessEqual,    // -le
    NumericGreater,      // -gt
    NumericGreaterEqual, // -ge

    // File comparison
    FileNewer, // -nt
    FileOlder, // -ot
    FileSame,  // -ef
}

/// Unary test operators
#[derive(Debug, Clone, PartialEq)]
pub enum TestUnaryOperator {
    // File tests
    FileExists,      // -e
    FileRegular,     // -f
    FileDirectory,   // -d
    FileSymlink,     // -L, -h
    FileReadable,    // -r
    FileWritable,    // -w
    FileExecutable,  // -x
    FileNonEmpty,    // -s
    FileBlockDevice, // -b
    FileCharDevice,  // -c
    FileFifo,        // -p
    FileSocket,      // -S
    FileSticky,      // -k
    FileSetgid,      // -g
    FileSetuid,      // -u
    FileOwned,       // -O
    FileGroupOwned,  // -G
    FileModified,    // -N
    FileTty,         // -t

    // String tests
    StringEmpty,    // -z
    StringNonEmpty, // -n

    // Variable tests
    VariableSet,   // -v
    VariableArray, // -a (bash extension)
}

/// Parameter expansion modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum ParameterModifier<'src> {
    // Default values
    UseDefault(&'src str),           // :-
    AssignDefault(&'src str),        // :=
    ErrorIfUnset(Option<&'src str>), // :?
    UseAlternative(&'src str),       // :+

    // Substring
    Substring {
        start: Box<AstNode<'src>>,
        length: Option<Box<AstNode<'src>>>,
    },

    // Pattern matching
    RemoveSmallestPrefix(&'src str), // #
    RemoveLargestPrefix(&'src str),  // ##
    RemoveSmallestSuffix(&'src str), // %
    RemoveLargestSuffix(&'src str),  // %%

    // Pattern replacement
    ReplaceFirst {
        pattern: &'src str,
        replacement: Option<&'src str>,
    },
    ReplaceAll {
        pattern: &'src str,
        replacement: Option<&'src str>,
    },

    // Case modification
    UppercaseFirst(&'src str), // ^
    UppercaseAll(&'src str),   // ^^
    LowercaseFirst(&'src str), // ,
    LowercaseAll(&'src str),   // ,,

    // Length
    Length, // #var
}

/// Case statement arms
#[derive(Debug, Clone, PartialEq)]
pub struct CaseArm<'src> {
    pub patterns: Vec<Pattern<'src>>,
    pub body: AstNode<'src>,
}

/// Match statement arms (modern feature)
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm<'src> {
    pub pattern: Pattern<'src>,
    pub guard: Option<AstNode<'src>>,
    pub body: AstNode<'src>,
}

/// Catch clause for try statements
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause<'src> {
    pub variable: Option<&'src str>,
    pub body: Box<AstNode<'src>>,
}

/// Function parameters
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter<'src> {
    pub name: &'src str,
    pub default: Option<AstNode<'src>>,
    pub is_variadic: bool,
}

/// Array elements
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayElement<'src> {
    pub index: Option<AstNode<'src>>,
    pub value: AstNode<'src>,
}

/// Advanced patterns for modern pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern<'src> {
    // Basic patterns
    Literal(&'src str),
    Variable(&'src str),
    Wildcard,

    // Glob patterns
    Glob(GlobPattern<'src>),

    // Range patterns
    Range {
        start: &'src str,
        end: &'src str,
    },

    // Alternative patterns (or)
    Alternative(Vec<Pattern<'src>>),

    // Rust-like patterns
    Tuple(Vec<Pattern<'src>>),
    Array(Vec<Pattern<'src>>),
    ArraySlice {
        before: Vec<Pattern<'src>>,
        rest: Option<Box<Pattern<'src>>>,
        after: Vec<Pattern<'src>>,
    },
    Object {
        fields: Vec<ObjectField<'src>>,
        rest: bool, // .. pattern
    },

    // Type-based patterns
    Type {
        type_name: &'src str,
        inner: Option<Box<Pattern<'src>>>,
    },

    // Guard patterns
    Guard {
        pattern: Box<Pattern<'src>>,
        condition: Box<AstNode<'src>>,
    },

    // Binding patterns
    Binding {
        name: &'src str,
        pattern: Box<Pattern<'src>>,
    },

    // Or-patterns (multiple patterns for same arm)
    Or(Vec<Pattern<'src>>),

    // Reference patterns
    Reference(Box<Pattern<'src>>),

    // Placeholder pattern
    Placeholder, // _
}

/// Object field patterns for destructuring
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectField<'src> {
    pub key: &'src str,
    pub pattern: Option<Pattern<'src>>, // None means field: field shorthand
    pub default: Option<AstNode<'src>>,
}

/// Pattern matching context for evaluation
#[derive(Debug, Clone, PartialEq)]
pub struct MatchContext<'src> {
    pub bindings: Vec<(&'src str, AstNode<'src>)>,
    pub matched: bool,
    pub exhaustive: bool,
}

/// Pattern matching expression with advanced features
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpression<'src> {
    pub expr: Box<AstNode<'src>>,
    pub arms: Vec<MatchArm<'src>>,
    pub is_exhaustive: bool,
    pub default_arm: Option<Box<AstNode<'src>>>,
}

/// Glob patterns
#[derive(Debug, Clone, PartialEq)]
pub struct GlobPattern<'src> {
    pub elements: Vec<GlobElement<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GlobElement<'src> {
    Literal(&'src str),
    Wildcard,   // *
    SingleChar, // ?
    CharacterClass {
        negated: bool,
        ranges: Vec<CharacterRange>,
    },
    BraceExpansion(Vec<&'src str>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterRange {
    pub start: char,
    pub end: Option<char>, // None for single character
}

/// Brace expansion elements
#[derive(Debug, Clone, PartialEq)]
pub enum BraceElement<'src> {
    Literal(&'src str),
    Sequence {
        start: i64,
        end: i64,
        step: Option<i64>,
    },
}

/// Process substitution direction
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessSubstitutionDirection {
    Input,  // <()
    Output, // >()
}

/// Quote types
#[derive(Debug, Clone, PartialEq)]
pub enum QuoteType {
    Single, // '...'
    Double, // "..."
    AnsiC,  // $'...'
    Locale, // $"..."
}

/// Number types
#[derive(Debug, Clone, PartialEq)]
pub enum NumberType {
    Decimal,
    Hexadecimal,
    Octal,
    Binary,
    Float,
}

/// Import types
#[derive(Debug, Clone, PartialEq)]
pub enum ImportType<'src> {
    Use { alias: Option<&'src str> },
    From { items: Vec<&'src str> },
    All,
}

/// Module paths
#[derive(Debug, Clone, PartialEq)]
pub struct ModulePath<'src> {
    pub segments: Vec<&'src str>,
}

/// Visibility modifiers
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Restricted(Vec<String>), // pub(crate), pub(super), etc.
}

/// Type definitions (for modern shell features)
#[derive(Debug, Clone, PartialEq)]
pub enum TypeDefinition<'src> {
    Struct { fields: Vec<FieldDefinition<'src>> },
    Enum { variants: Vec<EnumVariant<'src>> },
    Alias { target: &'src str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition<'src> {
    pub name: &'src str,
    pub type_name: &'src str,
    pub default: Option<AstNode<'src>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant<'src> {
    pub name: &'src str,
    pub fields: Option<Vec<FieldDefinition<'src>>>,
}

/// Source location information
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
    pub length: Option<u32>,
    pub file: Option<String>,
}

impl<'src> AstNode<'src> {
    /// Check if this node is a statement (can appear at statement level)
    pub fn is_statement(&self) -> bool {
        matches!(
            self,
            AstNode::Command { .. }
                | AstNode::Pipeline { .. }
                | AstNode::If { .. }
                | AstNode::For { .. }
                | AstNode::ForC { .. }
                | AstNode::While { .. }
                | AstNode::Until { .. }
                | AstNode::Case { .. }
                | AstNode::Select { .. }
                | AstNode::Match { .. }
                | AstNode::Try { .. }
                | AstNode::FunctionDeclaration { .. }
                | AstNode::MacroDeclaration { .. }
                | AstNode::VariableAssignment { .. }
                | AstNode::ArrayAssignment { .. }
                | AstNode::Return(_)
                | AstNode::Break(_)
                | AstNode::Continue(_)
                | AstNode::Exit(_)
                | AstNode::ImportStatement { .. }
                | AstNode::ModuleDeclaration { .. }
                | AstNode::TypeDeclaration { .. }
                | AstNode::ThrowStatement(_)
                | AstNode::Subshell(_)
                | AstNode::BraceGroup(_)
        )
    }

    /// Check if this node is an expression
    pub fn is_expression(&self) -> bool {
        matches!(
            self,
            AstNode::BinaryExpression { .. }
                | AstNode::UnaryExpression { .. }
                | AstNode::PostfixExpression { .. }
                | AstNode::ConditionalExpression { .. }
                | AstNode::TestExpression { .. }
                | AstNode::TestBinary { .. }
                | AstNode::TestUnary { .. }
                | AstNode::VariableExpansion { .. }
                | AstNode::CommandSubstitution { .. }
                | AstNode::ArithmeticExpansion { .. }
                | AstNode::ProcessSubstitution { .. }
                | AstNode::PathnameExpansion { .. }
                | AstNode::BraceExpansion { .. }
                | AstNode::TildeExpansion { .. }
                | AstNode::Word(_)
                | AstNode::StringLiteral { .. }
                | AstNode::NumberLiteral { .. }
                | AstNode::Array(_)
                | AstNode::FunctionCall { .. }
                | AstNode::Closure { .. }
                | AstNode::MacroInvocation { .. }
                | AstNode::AwaitExpression(_)
                | AstNode::YieldExpression(_)
        )
    }

    /// Check if this node can be used as a command name
    pub fn can_be_command_name(&self) -> bool {
        matches!(
            self,
            AstNode::Word(_)
                | AstNode::StringLiteral { .. }
                | AstNode::VariableExpansion { .. }
                | AstNode::CommandSubstitution { .. }
                | AstNode::PathnameExpansion { .. }
                | AstNode::TildeExpansion { .. }
        )
    }

    /// Get the precedence of this node if it's a binary operator
    pub fn precedence(&self) -> Option<u8> {
        match self {
            AstNode::BinaryExpression { operator, .. } => Some(operator.precedence()),
            _ => None,
        }
    }

    /// Check if this node has side effects
    pub fn has_side_effects(&self) -> bool {
        match self {
            AstNode::Command { .. }
            | AstNode::Pipeline { .. }
            | AstNode::FunctionCall { .. }
            | AstNode::VariableAssignment { .. }
            | AstNode::ArrayAssignment { .. }
            | AstNode::CommandSubstitution { .. }
            | AstNode::ProcessSubstitution { .. }
            | AstNode::Return(_)
            | AstNode::Break(_)
            | AstNode::Continue(_)
            | AstNode::Exit(_)
            | AstNode::ThrowStatement(_) => true,

            AstNode::BinaryExpression { left, right, .. } => {
                left.has_side_effects() || right.has_side_effects()
            }
            AstNode::UnaryExpression { operand, .. } => operand.has_side_effects(),
            AstNode::ConditionalExpression {
                condition,
                then_expr,
                else_expr,
            } => {
                condition.has_side_effects()
                    || then_expr.has_side_effects()
                    || else_expr.has_side_effects()
            }
            AstNode::Array(elements) => elements.iter().any(|e| e.has_side_effects()),

            _ => false,
        }
    }
}

impl BinaryOperator {
    /// Get the precedence of this binary operator
    pub fn precedence(&self) -> u8 {
        match self {
            BinaryOperator::LogicalOr => 1,
            BinaryOperator::LogicalAnd => 2,
            BinaryOperator::BitwiseOr => 3,
            BinaryOperator::BitwiseXor => 4,
            BinaryOperator::BitwiseAnd => 5,
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Match
            | BinaryOperator::NotMatch => 6,
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => 7,
            BinaryOperator::LeftShift | BinaryOperator::RightShift => 8,
            BinaryOperator::Add | BinaryOperator::Subtract => 9,
            BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => 10,
            BinaryOperator::Power => 11,
        }
    }

    /// Check if this operator is left-associative
    pub fn is_left_associative(&self) -> bool {
        !matches!(self, BinaryOperator::Power)
    }
}

impl fmt::Display for AstNode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AstNode::Word(s) => write!(f, "{s}"),
            AstNode::StringLiteral { value, .. } => write!(f, "\"{value}\""),
            AstNode::NumberLiteral { value, .. } => write!(f, "{value}"),
            AstNode::VariableExpansion { name, .. } => write!(f, "${name}"),
            AstNode::Command { name, args, .. } => {
                write!(f, "{name}")?;
                for arg in args {
                    write!(f, " {arg}")?;
                }
                Ok(())
            }
            AstNode::Pipeline { elements, .. } => {
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{element}")?;
                }
                Ok(())
            }
            _ => write!(f, "{self:?}"),
        }
    }
}
