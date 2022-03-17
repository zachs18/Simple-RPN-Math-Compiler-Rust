
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Item {
    FunctionItem(Function),
    StaticItem(Static),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Function {
    pub name: String,
    pub parameters: Vec<Variable>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Variable {
    pub name: String,
    pub mutable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Static {
    pub atomic: bool,
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Statement {
    LetStatement(Let),
    ExpressionStatement(Expression),
    LoopStatement(Loop),
    ReturnStatement(Expression),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Let {
    pub variable: Variable,
    pub value: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Loop {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Expression {
    AssignExpression{
        rhs: Box<Expression>,
        op: Operator,
        lhs: Box<Expression>,
    },
    OrExpression{
        rhs: Box<Expression>,
        op: Operator,
        lhs: Box<Expression>,
    },
    AndExpression{
        rhs: Box<Expression>,
        op: Operator,
        lhs: Box<Expression>,
    },
    CompareExpression{
        rhs: Box<Expression>,
        op: Operator,
        lhs: Box<Expression>,
    },
    AddExpression{
        rhs: Box<Expression>,
        op: Operator,
        lhs: Box<Expression>,
    },
    MulExpression{
        rhs: Box<Expression>,
        op: Operator,
        lhs: Box<Expression>,
    },
    ParenExpression{
        expr: Box<Expression>,
    },
    CallExpression{
        function: Box<Expression>,
        args: Vec<Expression>
    },
    VariableExpression{
        name: String,
    },
    BlockExpression{
        block: Block,
    },
    IntLiteralExpression{
        literal: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Operator {
    pub operator: String,
    pub checked: bool,
}

#[cfg(feature = "parsing")]
mod parsing;
#[cfg(feature = "parsing")]
pub use parsing::parse;

#[cfg(feature = "parsing")]
mod lexing;
#[cfg(feature = "parsing")]
pub use lexing::lex;