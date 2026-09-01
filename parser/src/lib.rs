pub mod ast;

#[cfg(test)]
mod test;

use ast::{BindKind, Conditional, Expr, KV, Node, Operator, Param, PlaceExpr, Statement};
use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::prelude::*;
use chumsky::span::SimpleSpan;
use lexer::Token;

/// Parse a full program, i.e. zero or more top-level items (statements or
/// expressions). Comments are skipped between any two tokens, so comment-only
/// files parse to an empty program.
pub fn parser<'a, T>() -> impl Parser<'a, T, Vec<Node>, Err<Rich<'a, Token>>>
where
    T: ValueInput<'a, Token = Token, Span = SimpleSpan>,
{
    // ---- comment skipping ----
    // Comments may appear between any two significant tokens, so every token
    // consumption below goes through `tok` (or a skip-prefixed filter).
    let skip_comments = any()
        .filter(|t| matches!(t, Token::InlineComment(_) | Token::LineComment(_)))
        .ignored()
        .repeated();

    // Parse a single exact token, skipping any comments that precede it.
    // `skip_comments` is `Copy`, so the closure takes its own copy.
    let tok = move |t: Token| {
        skip_comments
            .ignore_then(any().filter(move |x| *x == t))
            .boxed()
    };

    // ---- local helpers ----
    // A step of a place expression after its identifier.
    enum PlaceSuffix {
        Field(String),
        Index(Node),
    }

    // A suffix applied to an already-parsed expression.
    enum PostfixSuffix {
        Access(String),
        Method(String),
        Call(Vec<Node>),
        Index(Node),
    }

    // `let`/`mut` bind a simple identifier; anything more complex becomes an
    // assignment (`let a.b = 3` is just an assignment to a.b).
    fn bind_or_assign(target: PlaceExpr, value: Node, kind: BindKind) -> Node {
        match target {
            PlaceExpr::Identifier(name) => Node::stmt(Statement::Bind {
                name,
                kind,
                value: Box::new(value),
            }),
            target => Node::stmt(Statement::Assign {
                target,
                value: Box::new(value),
            }),
        }
    }


    // ---- recursive handles ----
    let mut place = Recursive::declare();
    let mut expr = Recursive::declare();
    let mut block = Recursive::declare();
    let mut dict = Recursive::declare();
    let mut list = Recursive::declare();
    let mut unary = Recursive::declare();
    let mut pow = Recursive::declare();

    // ---- primitives ----
    let int = skip_comments
        .ignore_then(
            any()
                .filter(|t| matches!(t, Token::Integer(_)))
                .map(|t| match t {
                    Token::Integer(x) => Node::expr(Expr::Integer(x)),
                    _ => unreachable!(),
                }),
        )
        .labelled("integer");

    let bool = skip_comments
        .ignore_then(
            any()
                .filter(|t| matches!(t, Token::Bool(_)))
                .map(|t| match t {
                    Token::Bool(x) => Node::expr(Expr::Bool(x)),
                    _ => unreachable!(),
                }),
        )
        .labelled("boolean");

    let string = skip_comments
        .ignore_then(
            any()
                .filter(|t| matches!(t, Token::String(_)))
                .map(|t| match t {
                    Token::String(x) => Node::expr(Expr::String(x)),
                    _ => unreachable!(),
                }),
        )
        .labelled("string");

    let ident = skip_comments
        .ignore_then(
            any()
                .filter(|t| matches!(t, Token::Ident(_)))
                .map(|t| match t {
                    Token::Ident(x) => Node::place(PlaceExpr::Identifier(x)),
                    _ => unreachable!(),
                }),
        )
        .labelled("identifier");

    let ident_as_str = skip_comments
        .ignore_then(
            any()
                .filter(|t| matches!(t, Token::Ident(_)))
                .map(|t| match t {
                    Token::Ident(x) => x,
                    _ => unreachable!(),
                }),
        )
        .labelled("identifier as string")
        .boxed();

    // ---- place expressions (assignment targets) ----
    // `a`, `a.b.c`, `obj::method`, `a[3]`
    let place_suffix = choice((
        tok(Token::Access)
            .ignore_then(ident_as_str.clone())
            .map(PlaceSuffix::Field),
        tok(Token::DoubleColon)
            .ignore_then(ident_as_str.clone())
            .map(PlaceSuffix::Field),
        tok(Token::OpenList)
            .ignore_then(expr.clone())
            .then_ignore(tok(Token::CloseList))
            .map(PlaceSuffix::Index),
    ));

    place.define(
        ident_as_str
            .clone()
            .map(PlaceExpr::Identifier)
            .foldl(place_suffix.repeated(), |base, suffix| match suffix {
                PlaceSuffix::Field(field) => PlaceExpr::Access {
                    base: Box::new(Node::place(base)),
                    field,
                },
                PlaceSuffix::Index(index) => PlaceExpr::Index {
                    base: Box::new(Node::place(base)),
                    index: Box::new(index),
                },
            })
            .boxed(),
    );

    // ---- object literals ----
    // `{ key = value, :method = fn(...), ... }`
    let key = tok(Token::Method)
        .ignore_then(ident_as_str.clone())
        .map(|k| (k, true))
        .or(ident_as_str.clone().map(|k| (k, false)));

    let kv = key
        .then_ignore(tok(Token::Assign))
        .then(expr.clone())
        .map(|((key, method), value)| KV {
            key,
            method,
            value: Box::new(value),
        })
        .labelled("kv pair");

    let kvset = kv
        .separated_by(tok(Token::ItemSep))
        .allow_trailing()
        .collect::<Vec<_>>()
        .labelled("dict fields");

    dict.define(
        tok(Token::OpenBlock)
            .ignore_then(kvset)
            .then_ignore(tok(Token::CloseBlock))
            .map(|kvs| Node::expr(Expr::Dict(kvs)))
            .boxed(),
    );

    // ---- list literals ----
    list.define(
        tok(Token::OpenList)
            .ignore_then(
                expr.clone()
                    .separated_by(tok(Token::ItemSep))
                    .allow_trailing()
                    .collect::<Vec<_>>(),
            )
            .then_ignore(tok(Token::CloseList))
            .map(|items| Node::expr(Expr::List(items)))
            .boxed(),
    );

    // ---- bindings & control-flow statements ----
    let r#let = tok(Token::Let)
        .ignore_then(place.clone())
        .then_ignore(tok(Token::Assign))
        .then(expr.clone())
        .then_ignore(tok(Token::Semicolon).or_not())
        .map(|(target, value)| bind_or_assign(target, value, BindKind::Borrow));

    let r#mut = tok(Token::Mut)
        .ignore_then(place.clone())
        .then_ignore(tok(Token::Assign))
        .then(expr.clone())
        .then_ignore(tok(Token::Semicolon).or_not())
        .map(|(target, value)| bind_or_assign(target, value, BindKind::BorrowMut));

    let r#assign = place
        .clone()
        .then_ignore(tok(Token::Assign))
        .then(expr.clone())
        .then_ignore(tok(Token::Semicolon).or_not())
        .map(|(target, value)| {
            Node::stmt(Statement::Assign {
                target,
                value: Box::new(value),
            })
        });

    let r#continue = tok(Token::Continue)
        .ignore_then(tok(Token::Semicolon).or_not())
        .map(|_| Node::stmt(Statement::Continue));

    let r#break = tok(Token::Break)
        .ignore_then(expr.clone().or_not())
        .then_ignore(tok(Token::Semicolon).or_not())
        .map(|value| Node::stmt(Statement::Break(value.map(Box::new))));

    let r#return = tok(Token::Return)
        .ignore_then(expr.clone().or_not())
        .then_ignore(tok(Token::Semicolon).or_not())
        .map(|value| Node::stmt(Statement::Return(value.map(Box::new))));

    // ---- function parameters ----
    // `&self`, `&mut x`, `x`, optionally with a default value
    let r#ref = tok(Token::Ref)
        .ignore_then(ident_as_str.clone())
        .then(tok(Token::Assign).ignore_then(expr.clone()).or_not())
        .map(|(name, default)| Param {
            name,
            default: default.map(Box::new),
            kind: BindKind::Borrow,
        });

    let r#mut_ref = tok(Token::Ref)
        .ignore_then(tok(Token::Mut))
        .ignore_then(ident_as_str.clone())
        .then(tok(Token::Assign).ignore_then(expr.clone()).or_not())
        .map(|(name, default)| Param {
            name,
            default: default.map(Box::new),
            kind: BindKind::BorrowMut,
        });

    let r#move = ident_as_str
        .clone()
        .then(tok(Token::Assign).ignore_then(expr.clone()).or_not())
        .map(|(name, default)| Param {
            name,
            default: default.map(Box::new),
            kind: BindKind::Move,
        });

    let r#fn_args = choice((r#ref, r#mut_ref, r#move))
        .separated_by(tok(Token::ItemSep))
        .collect::<Vec<_>>()
        .delimited_by(tok(Token::OpenParen), tok(Token::CloseParen))
        .labelled("fn arguments")
        .boxed();

    // ---- functions ----
    // Bodies are either a block or a single expression: `fn(x) x * 2`
    let r#fn_body = block.clone().or(expr.clone()).boxed();

    let r#fn_named = tok(Token::Function)
        .ignore_then(ident_as_str.clone())
        .then(
            r#fn_args
                .clone()
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>()
                .labelled("fn formal parameters"),
        )
        .then(r#fn_body.clone())
        .map(|((name, groups), body)| {
            Node::stmt(Statement::Fn(
                name,
                groups.into_iter().flatten().collect(),
                Box::new(body),
            ))
        });

    let r#fn_anon = tok(Token::Function)
        .ignore_then(
            r#fn_args
                .clone()
                .repeated()
                .at_least(1)
                .collect::<Vec<_>>()
                .labelled("fn anonymous parameters"),
        )
        .then(r#fn_body.clone())
        .map(|(groups, body)| {
            Node::expr(Expr::Fn(
                groups.into_iter().flatten().collect(),
                Box::new(body),
            ))
        });

    // ---- control-flow expressions (loop / while / for / if) ----
    let r#loop = tok(Token::Loop)
        .ignore_then(block.clone())
        .map(|body| Node::expr(Expr::Loop(Box::new(body))))
        .boxed();

    let r#while = tok(Token::While)
        .ignore_then(expr.clone())
        .then(block.clone())
        .map(|(cond, body)| Node::expr(Expr::While(Box::new(cond), Box::new(body))))
        .boxed();

    let r#for = tok(Token::For)
        .ignore_then(ident_as_str.clone())
        .then_ignore(tok(Token::In))
        .then(expr.clone())
        .then(choice((
            tok(Token::RangeInclusive).to(true),
            tok(Token::Range).to(false),
        )))
        .then(expr.clone())
        .then(block.clone())
        .map(|((((var, from), inclusive), to), body)| {
            Node::expr(Expr::For {
                var,
                from: Box::new(from),
                to: Box::new(to),
                inclusive,
                body: Box::new(body),
            })
        })
        .boxed();

    // `if (cond) { ... } else if (cond) { ... } else { ... }`
    //
    // An if-arm is `if (cond) { body }`. Arms chain through `else`, and the
    // chain ends with an optional bare `else { body }`, which is treated as an
    // arm whose condition is always true.
    let ifelse = tok(Token::If)
        .ignore_then(
            expr.clone()
                .labelled("if condition")
                .delimited_by(tok(Token::OpenParen), tok(Token::CloseParen)),
        )
        .then(block.clone().labelled("if body"))
        .map(|(cond, body)| Conditional {
            cond: Box::new(cond),
            eval: Box::new(body),
        })
        .labelled("if arm")
        .separated_by(tok(Token::Else))
        .at_least(1)
        .collect::<Vec<_>>()
        .then(
            tok(Token::Else)
                .ignore_then(
                    block
                        .clone()
                        .labelled("else body")
                        .or_not()
                        .map(|body| Conditional {
                            // A bare `else` is a condition that is always true.
                            cond: Box::new(Node::expr(Expr::Bool(true))),
                            eval: Box::new(body.unwrap_or_else(|| {
                                Node::expr(Expr::Block(Vec::new(), None))
                            })),
                        }),
                )
                .or_not(),
        )
        .map(|(mut arms, els)| {
            if let Some(els) = els {
                arms.push(els);
            }
            Node::expr(Expr::If(arms))
        })
        .boxed();

    // ---- atoms ----
    let atom = choice((
        int,
        bool,
        string,
        ident,
        dict.clone(),
        list.clone(),
        block.clone(),
        r#fn_anon,
        tok(Token::OpenParen)
            .ignore_then(expr.clone())
            .then_ignore(tok(Token::CloseParen))
            .labelled("grouped expression"),
    ));

    // ---- postfix: access, method slots, indexing, calls ----
    let call_args = tok(Token::OpenParen)
        .ignore_then(
            expr.clone()
                .separated_by(tok(Token::ItemSep))
                .collect::<Vec<_>>(),
        )
        .then_ignore(tok(Token::CloseParen));

    let postfix_suffix = choice((
        tok(Token::Access)
            .ignore_then(ident_as_str.clone())
            .map(PostfixSuffix::Access),
        // Both `x:method` (bound) and `x::method` (late-defined slot) surface as methods.
        tok(Token::Method)
            .ignore_then(ident_as_str.clone())
            .map(PostfixSuffix::Method),
        tok(Token::DoubleColon)
            .ignore_then(ident_as_str.clone())
            .map(PostfixSuffix::Method),
        call_args.map(PostfixSuffix::Call),
        tok(Token::OpenList)
            .ignore_then(expr.clone())
            .then_ignore(tok(Token::CloseList))
            .map(PostfixSuffix::Index),
    ));

    let postfix = atom
        .foldl(postfix_suffix.repeated(), |base, suffix| match suffix {
            PostfixSuffix::Access(field) => Node::place(PlaceExpr::Access {
                base: Box::new(base),
                field,
            }),
            PostfixSuffix::Method(name) => Node::expr(Expr::Method(Box::new(base), name)),
            PostfixSuffix::Call(args) => Node::expr(Expr::Call(Box::new(base), args)),
            PostfixSuffix::Index(index) => Node::place(PlaceExpr::Index {
                base: Box::new(base),
                index: Box::new(index),
            }),
        })
        .boxed();

    // ---- operators, lowest to highest precedence ----
    // `**` binds tighter than `*`/`/` and is right-associative
    pow.define(
        postfix
            .clone()
            .then(tok(Token::Pow).ignore_then(pow.clone()).or_not())
            .map(|(left, right)| match right {
                Some(right) => Node::binop(Operator::Exponent, left, right),
                None => left,
            })
            .boxed(),
    );

    unary.define(
        choice((
            tok(Token::Sub)
                .ignore_then(unary.clone())
                .map(|v| Node::unop(Operator::Minus, v)),
            pow.clone(),
        ))
        .boxed(),
    );

    let mul = unary
        .clone()
        .foldl(
            choice((
                tok(Token::Mul).to(Operator::Multiply),
                tok(Token::Div).to(Operator::Divide),
            ))
            .then(unary.clone())
            .repeated(),
            |l, (op, r)| Node::binop(op, l, r),
        )
        .boxed();

    let add = mul
        .clone()
        .foldl(
            choice((
                tok(Token::Add).to(Operator::Plus),
                tok(Token::Sub).to(Operator::Minus),
            ))
            .then(mul.clone())
            .repeated(),
            |l, (op, r)| Node::binop(op, l, r),
        )
        .boxed();

    // `>=`, `<=`, `>` and `<` arrive as identifiers from the lexer
    let cmp_op = any()
        .filter(|t| matches!(t, Token::Ident(s) if matches!(s.as_str(), ">=" | "<=" | ">" | "<")))
        .map(|t| match t {
            Token::Ident(s) => match s.as_str() {
                ">=" => Operator::Ge,
                "<=" => Operator::Le,
                ">" => Operator::Gt,
                "<" => Operator::Lt,
                _ => unreachable!(),
            },
            _ => unreachable!(),
        })
        .labelled("comparison operator");

    let cmp = add
        .clone()
        .foldl(
            choice((
                tok(Token::Eq).to(Operator::Eq),
                tok(Token::Neq).to(Operator::Neq),
                cmp_op,
            ))
            .then(add.clone())
            .repeated(),
            |l, (op, r)| Node::binop(op, l, r),
        )
        .boxed();

    let and = cmp
        .clone()
        .foldl(
            tok(Token::And).ignore_then(cmp.clone()).repeated(),
            |l, r| Node::binop(Operator::And, l, r),
        )
        .boxed();

    let or = and
        .clone()
        .foldl(
            tok(Token::Or).ignore_then(and.clone()).repeated(),
            |l, r| Node::binop(Operator::Or, l, r),
        )
        .boxed();

    expr.define(or);

    // ---- blocks & statements ----
    let statement = choice((
        r#let, r#mut, r#assign, r#fn_named, r#break, r#continue, r#return,
    ));

    // Semicolon-terminated items (block content), plus control flow which
    // terminates on its own closing brace.
    let item_semi = choice((
        statement,
        ifelse.clone(),
        r#loop.clone(),
        r#while.clone(),
        r#for.clone(),
        expr.clone().then_ignore(tok(Token::Semicolon)),
    ))
    .boxed();

    // Top-level items may also be bare expressions (no trailing semicolon).
    let item = choice((item_semi.clone(), expr.clone())).boxed();

    block.define(
        tok(Token::OpenBlock)
            .ignore_then(
                item_semi
                    .repeated()
                    .collect::<Vec<_>>()
                    .then(expr.clone().or_not()),
            )
            .then_ignore(tok(Token::CloseBlock))
            .map(|(items, tail)| Node::expr(Expr::Block(items, tail.map(Box::new))))
            .boxed(),
    );

    // Skip comments before, between and after items so that comment-only
    // files (and comments separating two items) parse cleanly.
    skip_comments
        .ignore_then(
            skip_comments
                .ignore_then(item)
                .repeated()
                .collect::<Vec<_>>(),
        )
        .then_ignore(skip_comments)
}

