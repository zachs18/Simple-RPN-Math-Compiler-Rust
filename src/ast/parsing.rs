use std::borrow::Cow;

use nom::{
    IResult,
    Parser,
    multi::{many0, separated_list0, separated_list1},
    branch::alt,
    combinator::{recognize, map_res, opt},
    sequence::{pair, tuple, delimited, terminated, preceded}, Err, error::ParseError,
};

use super::{*, lexing::{Token, Keyword}};

#[derive(Debug)]
pub enum Error<I> {
    Nom(nom::error::Error<I>),
    Other(Cow<'static, str>),
}

impl<I> ParseError<I> for Error<I> {
    fn from_error_kind(input: I, kind: nom::error::ErrorKind) -> Self {
        Self::Nom(ParseError::from_error_kind(input, kind))
    }

    fn append(input: I, kind: nom::error::ErrorKind, other: Self) -> Self {
        match other {
            Self::Other(msg) => Self::Other(msg),
            Self::Nom(nom) => Self::Nom(ParseError::append(input, kind, nom)),
        }
    }
}

type TokResult<'a, 'b, T> = IResult<&'a [Token<'b>], T, Error<&'a [Token<'b>]>>;

fn ident<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, &'b str> {
    match input.split_first() {
        Some((Token::Identifier(id), rest)) => Ok((rest, id)),
        None => Err(Err::Error(ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))),
        _ => Err(Err::Error(Error::Other("expected identifier".into()))),
    }
}

fn keyword(kw: Keyword) -> impl for<'a, 'b> Fn(&'a[Token<'b>]) -> TokResult<'a, 'b, Keyword> {
    move |input| {
        match input.split_first() {
            Some((Token::Keyword(k, _), rest)) if *k == kw => Ok((rest, kw)),
            None => Err(Err::Error(ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            ))),
            _ => Err(Err::Error(Error::Other(format!("expected keyword: {:?}", kw).into()))),
        }
    }
}

fn punct(pu: &'static str) -> impl for<'a, 'b> Fn(&'a[Token<'b>]) -> TokResult<'a, 'b, &'static str> {
    move |input| {
        match input.split_first() {
            Some((Token::Punct(p), rest)) if *p == pu => Ok((rest, pu)),
            None => Err(Err::Error(ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            ))),
            _ => Err(Err::Error(Error::Other(format!("expected punct: {:?}", pu).into()))),
        }
    }
}

fn operator(op: Operator) -> impl for<'a, 'b> Fn(&'a[Token<'b>]) -> TokResult<'a, 'b, Operator> {
    move |input| {
        match input.split_first() {
            Some((Token::Operator(o, c), rest))
                if *o == op.operator && (op.checked == c.is_some()) => Ok((rest, op.clone())),
            None => Err(Err::Error(ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Eof,
            ))),
            _ => Err(Err::Error(Error::Other(format!("expected operator: {:?}", op).into()))),
        }
    }
}

fn int_literal<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, &'b str> {
    match input.split_first() {
        Some((Token::IntLiteral(lit), rest)) => Ok((rest, lit)),
        None => Err(Err::Error(ParseError::from_error_kind(
            input,
            nom::error::ErrorKind::Eof,
        ))),
        _ => Err(Err::Error(Error::Other("expected integer literal".into()))),
    }
}

macro_rules! make_operator_fn {
    ($name:ident: $ops:tt, $err:literal) => {
        fn $name<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Operator> {
            const OPS: &[&str] = &$ops;
            match input.split_first() {
                Some((Token::Operator(o, c), rest)) => {
                    if OPS.contains(o) {
                        return Ok((rest, Operator{
                            operator: o.to_string(),
                            checked: c.is_some(),
                        }));
                    }
                    Err(Err::Error(Error::Other(format!("expected {} operator: {:?}", $err, OPS).into())))
                },
                None => Err(Err::Error(ParseError::from_error_kind(
                    input,
                    nom::error::ErrorKind::Eof,
                ))),
                _ => Err(Err::Error(Error::Other(format!("expected {} operator: {:?}", $err, OPS).into()))),
            }
        }
    };
}

make_operator_fn!(assign_op: ["=", "+=", "-=", "*=", "/=", "%="], "assignment");
make_operator_fn!(compare_op: [">", ">=", "<", "<=", "==", "!="], "comparison");
make_operator_fn!(and_op: ["&&"], "and");
make_operator_fn!(or_op: ["||"], "or");
make_operator_fn!(add_op: ["+", "-"], "addition-precedence");
make_operator_fn!(mul_op: ["*", "%", "/"], "multiplication-precedence");

pub fn parse<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Module> {
    many0(alt((
        function.map(|f| Item::FunctionItem(f)),
        static_.map(|s| Item::StaticItem(s)),
    ))).map(|items| Module { items })
    .parse(input)
}

fn function<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Function> {
    tuple((
        keyword(Keyword::Fn),
        ident,
        punct("("),
        param_list,
        punct(")"),
        block,
    )).map(|(_, name, _, parameters, _, body)| {
        Function{ name: name.to_owned(), parameters, body }
    }).parse(input)
}

fn param_list<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Vec<Variable>> {
    separated_list0(
        punct(","),
        variable,
    )(input)
}

fn variable<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Variable> {
    opt(keyword(Keyword::Mut)).and(ident).map(|(is_mut, id)| {
        Variable{ name: id.to_owned(), mutable: is_mut.is_some() }
    }).parse(input)
}

fn block<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Block> {
    delimited(punct("{"), many0(statement), punct("}"))
        .map(|statements| Block{ statements })
        .parse(input)
}

fn statement<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Statement> {
    alt((
        let_stmt.map(Statement::LetStatement),
        loop_stmt.map(Statement::LoopStatement),
        return_stmt.map(Statement::ReturnStatement),
        expr_stmt.map(Statement::ExpressionStatement),
    ))(input)
}

fn let_stmt<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Let> {
    tuple((
        keyword(Keyword::Let),
        variable,
        operator(Operator { operator: "=".to_owned(), checked: false }),
        expr,
        punct(";"),
    )).map(|(_, variable, _, value, _)| {
        Let { variable, value }
    }).parse(input)
}

fn loop_stmt<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Loop> {
    tuple((
        keyword(Keyword::While),
        expr,
        block,
    )).map(|(_, condition, body)| {
        Loop { condition, body }
    }).parse(input)
}

fn return_stmt<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Expression> {
    delimited(keyword(Keyword::Return), expr, punct(";"))(input)
}

fn expr_stmt<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Expression> {
    terminated(expr, punct(";"))(input)
}

enum Associativity {
    LeftToRight,
    RightToLeft,
    NonAssociative,
}

macro_rules! make_simple_subexpr_fn {
    ($name:ident: $inner:ident $op:ident $assoc:expr => $variant:ident) => {
        fn $name<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Expression> {
            tuple((
                $inner,
                opt(pair($op, $name)),
            )).map(|(lhs, rhs)| {
                use Associativity::*;
                match (rhs, $assoc) {
                    (None, _) => lhs,
                    (Some((op, Expression::$variant { lhs: rhs_lhs, op: rhs_op, rhs: rhs_rhs })), RightToLeft) => {
                        // ((lhs + rhs.lhs) + rhs.rhs)
                        let new_lhs = Expression::$variant { lhs: lhs.into(), op, rhs: rhs_lhs }.into();
                        let new_rhs = rhs_rhs;
                        let new_op = rhs_op;
                        Expression::$variant { lhs: new_lhs, op: new_op, rhs: new_rhs }
                    }
                    (Some((op, Expression::$variant { .. })), NonAssociative) => {
                        todo!("handle parsing error here")
                    }
                    (Some((op, rhs)), _) => {
                        // Handles different precedence operators, as well as LeftToRight associative same-precedence operators
                        Expression::$variant { lhs: lhs.into(), op, rhs: rhs.into() }
                    }
                }
            }).parse(input)
        }
    };
}

fn expr<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Expression> {
    fn atom_expr<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Expression> {
        alt((
            delimited(punct("("), expr, punct(")")).map(|expr| Expression::ParenExpression { expr: expr.into() }),
            block.map(|block| Expression::BlockExpression{ block }),
            ident.map(|name| Expression::VariableExpression { name: name.to_owned() }),
            int_literal.map(|lit| Expression::IntLiteralExpression { literal: lit.into() }),
        ))(input)
    }
    fn call_expr<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Expression> {
        pair(atom_expr, call_expr_tail).map(|(func, argss)| {
            let mut expr = func;
            for arg_list in argss {
                expr = Expression::CallExpression { function: expr.into(), args: arg_list };
            }
            expr
        }).parse(input)
    }
    fn arg_list<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Vec<Expression>> {
        separated_list0(punct(","), expr)(input)
    }
    fn call_expr_tail<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Vec<Vec<Expression>>> {
        many0(delimited(punct("("), arg_list, punct(")")))(input)
    }
    use Associativity::*;
    make_simple_subexpr_fn!(mul_expr: call_expr mul_op LeftToRight => MulExpression);
    make_simple_subexpr_fn!(add_expr: mul_expr add_op  LeftToRight => AddExpression);
    make_simple_subexpr_fn!(compare_expr: add_expr compare_op NonAssociative => CompareExpression);
    make_simple_subexpr_fn!(and_expr: compare_expr and_op  LeftToRight => AndExpression);
    make_simple_subexpr_fn!(or_expr: and_expr or_op        LeftToRight => OrExpression);
    make_simple_subexpr_fn!(assign_expr: or_expr assign_op RightToLeft => AssignExpression);
    assign_expr(input)
}

fn static_<'a, 'b>(input: &'a[Token<'b>]) -> TokResult<'a, 'b, Static> {
    tuple((
        keyword(Keyword::Static),
        opt(keyword(Keyword::Atomic)),
        ident,
        assign_op,
        expr,
        punct(";"),
    )).map(|(_, atomic, name, _, value, _)| {
        Static{
            atomic: atomic.is_some(),
            name: name.to_owned(),
            value,
        }
    }).parse(input)
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test2() {
        let input = r#"
        fn test(a, mut b) {
            let mut acc = 1;
            while b >= 1 {
                acc = acc *? a;
                b -= 1;
            }
            return acc;
        }
        "#;
        let output = Module{
            items: vec![Item::FunctionItem(Function{
                name: "test".into(),
                parameters: vec![Variable{ name: "a".into(), mutable: false }, Variable{ name: "b".into(), mutable: true }],
                body: Block { statements: vec![
                    Statement::LetStatement(Let{ variable: Variable { name: "acc".into(), mutable: true },  value: Expression::IntLiteralExpression { literal: "1".into() } }),
                    Statement::LoopStatement(Loop{
                        condition: Expression::CompareExpression {
                            lhs: Expression::VariableExpression { name: "b".into() }.into(),
                            op: Operator{ operator: ">=".into(), checked: false },
                            rhs: Expression::IntLiteralExpression { literal: "1".into() }.into(),
                        },
                        body: Block { statements: vec![
                            Statement::ExpressionStatement(Expression::AssignExpression{
                                lhs: Expression::VariableExpression { name: "acc".into() }.into(),
                                op: Operator{ operator: "=".into(), checked: false },
                                rhs: Expression::MulExpression {
                                    lhs: Expression::VariableExpression { name: "acc".into() }.into(),
                                    op: Operator{ operator: "*".into(), checked: true },
                                    rhs: Expression::VariableExpression { name: "a".into() }.into(),
                                }.into(),
                            }),
                            Statement::ExpressionStatement(Expression::AssignExpression{
                                lhs: Expression::VariableExpression { name: "b".into() }.into(),
                                op: Operator{ operator: "-=".into(), checked: false },
                                rhs: Expression::IntLiteralExpression { literal: "1".into() }.into(),
                            }),
                        ] }
                    }),
                    Statement::ReturnStatement(Expression::VariableExpression { name: "acc".into() }),
                ] },
            })]
        };
        let tokens = lex(input).unwrap().1;
        let result = parse(&tokens).unwrap().1;
        assert_eq!(result, output);
    }
    #[test]
    fn associativity_1() {
        let input = r#"
        b -? 1 - x
        "#;
        let output = Expression::AddExpression {
            lhs: Expression::VariableExpression { name: "b".into() }.into(),
            op: Operator{ operator: "-".into(), checked: true },
            rhs: Expression::AddExpression {
                lhs: Expression::IntLiteralExpression { literal: "1".into() }.into(),
                op: Operator{ operator: "-".into(), checked: false },
                rhs: Expression::VariableExpression { name: "x".into() }.into(),
            }.into(),
        };
        let tokens = lex(input).unwrap().1;
        let result = expr(&tokens).unwrap().1;
        assert_eq!(result, output);
    }
    #[test]
    fn associativity_2() {
        let input = r#"
        b -=? 1 = x
        "#;
        let output = Expression::AssignExpression {
            lhs: Expression::AssignExpression {
                lhs: Expression::VariableExpression { name: "b".into() }.into(),
                op: Operator{ operator: "-=".into(), checked: true },
                rhs: Expression::IntLiteralExpression { literal: "1".into() }.into(),
            }.into(),
            op: Operator{ operator: "=".into(), checked: false },
            rhs: Expression::VariableExpression { name: "x".into() }.into(),
        };
        let tokens = lex(input).unwrap().1;
        let result = expr(&tokens).unwrap().1;
        assert_eq!(result, output);
    }
    #[test]
    fn precedence_1() {
        let input = r#"
        b -? 1 = x
        "#;
        let output = Expression::AssignExpression {
            lhs: Expression::AddExpression {
                lhs: Expression::VariableExpression { name: "b".into() }.into(),
                op: Operator{ operator: "-".into(), checked: true },
                rhs: Expression::IntLiteralExpression { literal: "1".into() }.into(),
            }.into(),
            op: Operator{ operator: "=".into(), checked: false },
            rhs: Expression::VariableExpression { name: "x".into() }.into(),
        };
        let tokens = lex(input).unwrap().1;
        let result = expr(&tokens).unwrap().1;
        assert_eq!(result, output);
    }
    #[test]
    fn precedence_2() {
        let input = r#"
        b -=? 1 - x
        "#;
        let output = Expression::AssignExpression {
            lhs: Expression::VariableExpression { name: "b".into() }.into(),
            op: Operator{ operator: "-=".into(), checked: true },
            rhs: Expression::AddExpression {
                lhs: Expression::IntLiteralExpression { literal: "1".into() }.into(),
                op: Operator{ operator: "-".into(), checked: false },
                rhs: Expression::VariableExpression { name: "x".into() }.into(),
            }.into(),
        };
        let tokens = lex(input).unwrap().1;
        let result = expr(&tokens).unwrap().1;
        assert_eq!(result, output);
    }
}