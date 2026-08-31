pub mod ast;

use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use chumsky::input::ValueInput;
use chumsky::extra::Err;
use lexer::Token;
use ast::Node;

pub fn parser<'a, T>() -> impl Parser<'a, T, Node, Err<Rich<'a, Token>>>
where
    T: ValueInput<'a, Token=Token, Span=SimpleSpan>
{
    let mut expr = Recursive::declare();
    let mut block = Recursive::declare();
    let mut dict = Recursive::declare();
    let mut list = Recursive::declare();

    let int = any()
        .filter(|t| matches!(t, Token::Integer(_)))
        .map (|t| match t {
            Token::Integer(x) => x,
            _ => unreachable!()
        })
        .labelled("integer")
        .boxed();

    
    todo!()
}
