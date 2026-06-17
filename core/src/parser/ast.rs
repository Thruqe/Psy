#[derive(Debug, Clone)]
pub struct ModuleImport {
    pub name: String,
    pub functions: Option<Vec<String>>, // None = import everything
}

#[derive(Debug, Clone)]
pub enum Statement {
    Import {
        modules: Vec<ModuleImport>,
    },
    Assign {
        variable: String,
        expression: Expression,
    },
    Input {
        variables: Vec<String>,
    },
    Output {
        values: Vec<OutputValue>,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_if_branches: Vec<(Expression, Vec<Statement>)>,
        else_branch: Vec<Statement>,
    },
    ForLoop {
        variable: String,
        start: Expression,
        end: Expression,
        body: Vec<Statement>,
    },
    WhileLoop {
        condition: Expression,
        body: Vec<Statement>,
    },
    DeclareArray {
        name: String,
        size: usize,
    },
    ArrayAssign {
        name: String,
        index: Box<Expression>,
        value: Box<Expression>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
    ConstDeclaration {
        name: String,
        expression: Expression,
    },
    StaticDeclaration {
        name: String,
        expression: Expression,
    },
    ExpressionStatement(Expression),
    Return {
        value: Option<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    ArrayAccess {
        name: String,
        index: Box<Expression>,
    },
    ArrayLiteral(Vec<Expression>),
    BinaryOp {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },
    UnaryOp {
        operator: UnaryOperator,
        expr: Box<Expression>,
    },
    FunctionCall {
        name: String,
        arguments: Vec<Expression>,
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
    Expression(Expression),
    StringLiteral(String),
}
