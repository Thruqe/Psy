#[derive(Debug, Clone, PartialEq)]
pub struct Spanned<T> {
    pub node: T,
    pub line: usize,
    pub column: usize,
}

impl<T> Spanned<T> {
    pub fn new(node: T, line: usize, column: usize) -> Self {
        Spanned { node, line, column }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign {
        variable: String,
        expression: Spanned<Expression>,
    },
    Input {
        variables: Vec<String>,
    },
    Output {
        values: Vec<OutputValue>,
    },
    If {
        condition: Spanned<Expression>,
        then_branch: Vec<Spanned<Statement>>,
        else_if_branches: Vec<(Spanned<Expression>, Vec<Spanned<Statement>>)>,
        else_branch: Vec<Spanned<Statement>>,
    },
    ForLoop {
        variable: String,
        start: Spanned<Expression>,
        end: Spanned<Expression>,
        body: Vec<Spanned<Statement>>,
    },
    WhileLoop {
        condition: Spanned<Expression>,
        body: Vec<Spanned<Statement>>,
    },
    DeclareArray {
        name: String,
        size: usize,
    },
    ArrayAssign {
        name: String,
        index: Box<Spanned<Expression>>,
        value: Box<Spanned<Expression>>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Spanned<Statement>>,
    },
    Return {
        value: Option<Spanned<Expression>>,
    },
    Import {
        modules: Vec<ModuleImport>,
    },
    ConstDeclaration {
        name: String,
        expression: Spanned<Expression>,
    },
    StaticDeclaration {
        name: String,
        expression: Spanned<Expression>,
    },
    ExpressionStatement(Spanned<Expression>),
    Public(Box<Spanned<Statement>>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    ArrayAccess {
        name: String,
        index: Box<Spanned<Expression>>,
    },
    ArrayLiteral(Vec<Spanned<Expression>>),
    FunctionCall {
        name: String,
        arguments: Vec<Spanned<Expression>>,
    },
    BinaryOp {
        left: Box<Spanned<Expression>>,
        operator: Operator,
        right: Box<Spanned<Expression>>,
    },
    UnaryOp {
        operator: UnaryOperator,
        expr: Box<Spanned<Expression>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessEqual,
    GreaterEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OutputValue {
    Expression(Spanned<Expression>),
    StringLiteral(String),
}

#[derive(Debug, Clone)]
pub struct ModuleImport {
    pub name: String,
    pub functions: Option<Vec<String>>,
}
