pub mod ast;

use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use chumsky::input::ValueInput;
use chumsky::extra::Err;
use lexer::Token;
use ast::{Node, Expr, Param, Statement, BindKind};

pub fn parser<'a, T>() -> impl Parser<'a, T, Expr, Err<Rich<'a, Token>>>
where
    T: ValueInput<'a, Token=Token, Span=SimpleSpan>
{
    let mut expr = Recursive::declare();
    let mut place = Recursive::declare();
    let mut block = Recursive::declare();
    let mut dict = Recursive::declare();
    let mut list = Recursive::declare();

    let int = any()
        .filter(|t| matches!(t, Token::Integer(_)))
        .map (|t| match t {
            Token::Integer(x) => Expr::Integer(x),
            _ => unreachable!()
        })
        .labelled("integer")
        .boxed();

    let ident = any()
        .filter(|t| matches!(t, Token::Ident(_)))
        .map (|t| match t {
            Token::Ident(x) => Expr::Ident(x),
            _ => unreachable!()
        })
        .labelled("identifier")
        .boxed();

    let ident_as_str = any()
        .filter(|t| matches!(t, Token::Ident(_)))
        .map (|t| match t {
            Token::Ident(x) => x,
            _ => unreachable!()
        })
        .labelled("identifier as string")
        .boxed();

    let place = ident_as_str.clone(); // TODO: Implement place expression parser

    let kvset = place
        .clone()
        .then_ignore(just(Token::Assign))
        .then(expr.clone())
        .map(ast::KV)
        .labelled("kv pair")
        .separated_by(just(Token::ItemSep).labelled("comma sep"))
        .allow_trailing()
        .collect::<Vec<_>>()
        .labelled("dict fields")
        .boxed();

    let r#let = just(Token::Let)
        .labelled("let")
        .ignore_then(place.clone())
        .then_ignore(just(Token::Assign))
        .then(expr.clone())
        .then_ignore(just(Token::Semicolon))
        .map(|(place, val)| Expr::Assign(place, val));

    let r#mut = just(Token::Mut)
        .labelled("mut")
        .ignore_then(place.clone())
        .then_ignore(just(Token::Assign))
        .then(expr.clone())
        .then_ignore(just(Token::Semicolon))
        .map(|(place, val)| Expr::Assign(place, val));

    let r#continue = just(Token::Continue)
        .labelled("continue")
        .ignore_then(just(Token::Semicolon))
        .map(Expr::Continue)
        .boxed();

    let r#break = just(Token::Break)
        .labelled("break")
        .ignore_then(
            expr.clone()
                .map(|i| Box::new(i))
                .labelled("break body")
                .or_not())
        .map(|e| Expr::Break(e))
        .boxed();

    let r#return = just(Token::Return)
        .labelled("return")
        .ignore_then(
            expr.clone()
                .map(|i| Box::new(i))
                .labelled("return body")
                .or_not())
        .map(|e| Expr::Return(e))
        .boxed();

    let r#ref = just(Token::Ref)
        .labelled("ref")
        .ignore_then(ident.clone())
        .then(
            just(Token::Assign)
            .ignore_then(expr.clone())
            .labelled("ref assign")
            .or_not())
        .map(|(r, n)| Param {
            name: r.clone(),
            default: n,
            kind: BindKind::Borrow
        });

    let r#mut_ref = just(Token::Ref)
        .labelled("mut ref")
        .ignore_then(just(Token::Mut))
        .ignore_then(ident.clone())
        .then(
            just(Token::Assign)
            .ignore_then(expr.clone())
            .labelled("mut ref assign")
            .or_not())
        .map(|(r, n)| Param {
            name: r.clone(),
            default: n,
            kind: BindKind::BorrowMut
        });

    let r#move_ref = ident
        .clone()
        .labelled("move ref")
        .then(
            just(Token::Assign)
            .ignore_then(expr.clone())
            .labelled("move assign")
            .or_not())
        .map(|(r, n)| Param {
            name: r.clone(),
            default: n,
            kind: BindKind::Move
        });

    let r#fn_args = choice((
            r#ref,
            r#mut_ref,
            r#move_ref
        ))
        .separated_by(just(Token::ItemSep))
        .labelled("fn arguments")
        .delimited_by(Token::OpenParen, Token::CloseParen)
        .boxed();

    let r#fn_named = just(Token::Function)
        .ignore_then(ident_as_str.clone())
        .then(
            r#fn_args
            .repeated()
            .at_least(1)
            .labelled("fn formal parameters")
        )
        .then(block.clone())
        .map(Statement::Fn);

    let r#fn_anon = just(Token::Function)
        .ignore_then(
            r#fn_args
            .clone()
            .repeated()
            .at_least(1)
            .labelled("fn anonymous parameters")
        )
        .then(block.clone()) 
        .map(Expr::Fn);


    todo!()
}
