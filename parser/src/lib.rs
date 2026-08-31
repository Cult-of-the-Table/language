use chumsky::prelude::*;
use lexer::Token;

pub fn parser<'a, T>() -> impl Parser<'a, T, Expr, Err<Rich<'a, Token>>> + Clone
