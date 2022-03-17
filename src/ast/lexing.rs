use std::{collections::HashMap};
use nom::{
    IResult,
    Parser,
    bytes::complete::{tag},
    character::complete::{alpha1, alphanumeric1, multispace0, one_of},
    multi::{many0, separated_list0, many1},
    branch::alt,
    combinator::{recognize, map_res, opt, eof},
    sequence::{pair, tuple, delimited, terminated, preceded}, number,
};

macro_rules! make_keywords {
    ($(#[$attr:meta])*  $vis:vis enum $name:ident { 
        $($variant:ident = $text:literal),* $(,)?
    }) => {
        $(#[$attr])*
        $vis enum $name {
            $($variant),*
        }
        lazy_static::lazy_static! {
            static ref KEYWORDS: HashMap<&'static str, Keyword> = HashMap::from([
                $( ($text, $name::$variant) ),*
            ]);
        }
    }
}

make_keywords!{
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    While = "while",
    Fn = "fn",
    Let = "let",
    Return = "return",
    Atomic = "atomic",
    Mut = "mut",
    Static = "static",
}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token<'a> {
    Identifier(&'a str),
    Keyword(Keyword, &'a str),
    Operator(&'a str, Option<char>),
    Punct(&'a str),
    IntLiteral(&'a str),
}

pub fn lex(input: &str) -> IResult<&str, Vec<Token>> {
    delimited(multispace0, many0(
        terminated(alt((
            ident_or_kw,
            punct,
            operator,
            int_literal,
        )), multispace0)
    ), eof)(input)
}

fn ident_or_kw(input: &str) -> IResult<&str, Token> {
    let (rest, id) = recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"))))
    ))(input)?;
    match KEYWORDS.get(id) {
        Some(kw) => Ok((rest, Token::Keyword(*kw, id))),
        None => Ok((rest, Token::Identifier(id))),
    }
}

fn punct(input: &str) -> IResult<&str, Token> {
    recognize(one_of("(){};,")).map(Token::Punct).parse(input)
}

fn operator(input: &str) -> IResult<&str, Token> {
    pair(
        alt((
            tag("=="),
            tag("!="),
            tag(">="),
            tag("<="),
            tag("&&"),
            tag("||"),
            recognize(pair( // augmented assignment
                one_of("+-*%/"),
                tag("="),
            )),
            recognize(one_of("+-*%/=<>"))
        )),
        opt(nom::character::complete::char('?'))
    ).map(|(o, c)| Token::Operator(o, c)).parse(input)
}

fn int_literal(input: &str) -> IResult<&str, Token> {
    recognize(pair(
        opt(tag("-")),
        many1(one_of("0123456789"))
    )).map(|s| Token::IntLiteral(s)).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test1() {
        let input = "Hello 45 static world(x, u32) - return";
        let output: Vec<Token> = vec![
            Token::Identifier("Hello"),
            Token::IntLiteral("45"),
            Token::Keyword(Keyword::Static, "static"),
            Token::Identifier("world"),
            Token::Punct("("),
            Token::Identifier("x"),
            Token::Punct(","),
            Token::Identifier("u32"),
            Token::Punct(")"),
            Token::Operator("-", None),
            Token::Keyword(Keyword::Return, "return"),
        ];
        let result = lex(input).unwrap().1;
        assert_eq!(result, output);
    }
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
        let output: Vec<Token> = vec![
            Token::Keyword(Keyword::Fn, "fn"),
            Token::Identifier("test"),
            Token::Punct("("),
            Token::Identifier("a"),
            Token::Punct(","),
            Token::Keyword(Keyword::Mut, "mut"),
            Token::Identifier("b"),
            Token::Punct(")"),

            Token::Punct("{"),

                Token::Keyword(Keyword::Let, "let"),
                Token::Keyword(Keyword::Mut, "mut"),
                Token::Identifier("acc"),
                Token::Operator("=", None),
                Token::IntLiteral("1"),
                Token::Punct(";"),

                Token::Keyword(Keyword::While, "while"),
                Token::Identifier("b"),
                Token::Operator(">=", None),
                Token::IntLiteral("1"),

                Token::Punct("{"),

                    Token::Identifier("acc"),
                    Token::Operator("=", None),
                    Token::Identifier("acc"),
                    Token::Operator("*", Some('?')),
                    Token::Identifier("a"),
                    Token::Punct(";"),

                    Token::Identifier("b"),
                    Token::Operator("-=", None),
                    Token::IntLiteral("1"),
                    Token::Punct(";"),

                Token::Punct("}"),

                Token::Keyword(Keyword::Return, "return"),
                Token::Identifier("acc"),
                Token::Punct(";"),

            Token::Punct("}"),
        ];
        let result = lex(input).unwrap().1;
        assert_eq!(result, output);
    }
}