pub mod ast;

use ast::{Expr, Node, PlaceExpr, Statement};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use lexer::Token;

use crate::ast::BindKind;

pub fn parser<'a, T>() -> impl Parser<'a, T, Node, Err<Rich<'a, Token>>>
where
    T: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    let mut expr = Recursive::declare();
    let mut place = Recursive::declare();
    let mut block = Recursive::declare();
    let mut dict = Recursive::declare();
    let mut list = Recursive::declare();

    let int = any()
        .filter(|t| matches!(t, Token::Integer(_)))
        .map(|t| match t {
            Token::Integer(x) => Expr::Integer(x),
            _ => unreachable!(),
        })
        .labelled("integer")
        .boxed();

    let ident = any()
        .filter(|t| matches!(t, Token::Ident(_)))
        .map(|t| match t {
            Token::Ident(x) => PlaceExpr::Identifier(x),
            _ => unreachable!(),
        })
        .labelled("identifier")
        .boxed();

    let ident_as_str = any()
        .filter(|t| matches!(t, Token::Ident(_)))
        .map(|t| match t {
            Token::Ident(x) => x,
            _ => unreachable!(),
        })
        .labelled("identifier as string")
        .boxed();

    let place = ident_as_str.clone(); // TODO: Implement place expression parser

    let r#let = just(Token::Let)
        .labelled("let")
        .ignore_then(place.clone())
        .then_ignore(just(Token::Assign))
        .then(expr.clone())
        .then_ignore(just(Token::Semicolon))
        .map(|(name, val)| Node::bind(name, BindKind::Move, val));

    let r#mut = just(Token::Mut)
        .labelled("mut")
        .ignore_then(place.clone())
        .then_ignore(just(Token::Assign))
        .then(expr.clone())
        .then_ignore(just(Token::Semicolon))
        .map(|(name, val)| Node::bind(name, BindKind::BorrowMut, val));

    let r#continue = just(Token::Continue)
        .labelled("continue")
        .ignore_then(just(Token::Semicolon))
        .map(|_| Node::stmt(Statement::Continue))
        .boxed();

    let r#break = just(Token::Break)
        .labelled("break")
        .ignore_then(
            expr.clone()
                .map(|i| Box::new(i))
                .labelled("break body")
                .or_not(),
        )
        .map(|e| Statement::Break(e))
        .boxed();

    let r#return = just(Token::Return)
        .labelled("return")
        .ignore_then(
            expr.clone()
                .map(|i| Box::new(i))
                .labelled("return body")
                .or_not(),
        )
        .map(|e| Statement::Return(e))
        .boxed();
}
