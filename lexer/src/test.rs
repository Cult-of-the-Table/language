use super::*;

use logos::Logos;

// Samples live in `sample_code/*.ax` at the workspace root (outside the
// crates) so the same files can be reused by parser/runtime tests later.
// Programs are top-level code like Python — no `main` needed — so even a
// file whose entire content is one expression (e.g. `"foo"`) is valid; it
// just evals and finishes.
const LANGUAGE_REFERENCE: &str = include_str!("../../sample_code/language_reference.ax");
const LET_LOOP: &str = include_str!("../../sample_code/let_loop.ax");
const FULL_PROGRAM: &str = include_str!("../../sample_code/full_program.ax");

const STRING_LITERAL: &str = include_str!("../../sample_code/string_literal.ax");
const CHAR_LITERAL: &str = include_str!("../../sample_code/char_literal.ax");
const INTEGER_LITERAL: &str = include_str!("../../sample_code/integer_literal.ax");
const BOOLEAN_LITERAL: &str = include_str!("../../sample_code/boolean_literal.ax");
const ARITHMETIC: &str = include_str!("../../sample_code/arithmetic.ax");
const KEYWORD_TOUR: &str = include_str!("../../sample_code/keyword_tour.ax");
const LINE_COMMENT: &str = include_str!("../../sample_code/line_comment.ax");
const BLOCK_COMMENT: &str = include_str!("../../sample_code/block_comment.ax");
const COMMENTS_IN_CODE: &str = include_str!("../../sample_code/comments_in_code.ax");

/// Lex `src` into a flat `Vec<Token>`, panicking on unlexable input.
fn lex(src: &str) -> Vec<Token> {
    Token::lexer(src).map(|r| r.unwrap()).collect()
}

fn ident(s: &str) -> Token {
    Token::Ident(s.into())
}

fn string(s: &str) -> Token {
    Token::String(s.into())
}

fn int(n: i64) -> Token {
    Token::Integer(n)
}

// ----------------------------------------------------------------
// Whitespace
// ----------------------------------------------------------------

#[test]
fn all_whitespace_variants_are_skipped() {
    assert!(lex(" \t\n\r\x0C ").is_empty());
    assert_eq!(
        lex("loop\tif \nelse\r\x0C"),
        vec![Token::Loop, Token::If, Token::Else]
    );
}

// ----------------------------------------------------------------
// Keywords
// ----------------------------------------------------------------

/// `keyword_tour.ax` exercises every keyword as part of one top-level program.
#[test]
fn lexes_all_keywords() {
    assert_eq!(
        lex(KEYWORD_TOUR),
        vec![
            Token::Function,
            ident("keyword_tour"),
            Token::OpenParen,
            ident("limit"),
            Token::CloseParen,
            Token::OpenBlock,
            Token::Let,
            ident("total"),
            Token::Assign,
            int(0),
            Token::Semicolon,
            Token::Mut,
            ident("count"),
            Token::Assign,
            int(0),
            Token::Semicolon,
            Token::Loop,
            Token::OpenBlock,
            Token::If,
            Token::OpenParen,
            ident("count"),
            ident(">="),
            ident("limit"),
            Token::CloseParen,
            Token::OpenBlock,
            Token::Break,
            Token::Semicolon,
            Token::CloseBlock,
            ident("count"),
            Token::Assign,
            ident("count"),
            Token::Add,
            int(1),
            Token::Semicolon,
            Token::Continue,
            Token::Semicolon,
            Token::CloseBlock,
            Token::While,
            ident("count"),
            ident("<"),
            int(10),
            Token::OpenBlock,
            ident("count"),
            Token::Assign,
            ident("count"),
            Token::Add,
            int(1),
            Token::Semicolon,
            Token::CloseBlock,
            Token::For,
            ident("n"),
            Token::In,
            int(0),
            Token::Range,
            int(3),
            Token::OpenBlock,
            ident("total"),
            Token::Assign,
            ident("total"),
            Token::Add,
            ident("n"),
            Token::Semicolon,
            Token::CloseBlock,
            Token::Let,
            ident("ok"),
            Token::Assign,
            ident("count"),
            Token::Neq,
            int(0),
            Token::And,
            ident("total"),
            Token::Eq,
            int(0),
            Token::Or,
            ident("count"),
            Token::Eq,
            ident("limit"),
            Token::Semicolon,
            Token::If,
            Token::OpenParen,
            ident("ok"),
            Token::CloseParen,
            Token::OpenBlock,
            Token::Return,
            ident("total"),
            Token::Semicolon,
            Token::CloseBlock,
            Token::Else,
            Token::OpenBlock,
            Token::Return,
            int(0),
            Token::Semicolon,
            Token::CloseBlock,
            Token::CloseBlock,
        ]
    );
}

#[test]
fn longer_identifiers_beat_keyword_prefixes() {
    assert_eq!(lex("loopy"), vec![ident("loopy")]);
    assert_eq!(lex("iffy"), vec![ident("iffy")]);
    assert_eq!(lex("into"), vec![ident("into")]);
    assert_eq!(lex("android"), vec![ident("android")]);
    assert_eq!(lex("truex"), vec![ident("truex")]);
    assert_eq!(lex("letx"), vec![ident("letx")]);
    assert_eq!(lex("forloop"), vec![ident("forloop")]);
}

// ----------------------------------------------------------------
// Identifiers
// ----------------------------------------------------------------

#[test]
fn lexes_identifiers() {
    assert_eq!(
        lex("foo bar some_field _x"),
        vec![ident("foo"), ident("bar"), ident("some_field"), ident("_x")]
    );
}

#[test]
fn identifiers_may_contain_digits_after_the_first_char() {
    assert_eq!(
        lex("arg1 x5 foo2"),
        vec![ident("arg1"), ident("x5"), ident("foo2")]
    );
}

#[test]
fn identifiers_cannot_start_with_a_digit() {
    assert_eq!(lex("123abc"), vec![int(123), ident("abc")]);
    assert_eq!(lex("0x"), vec![int(0), ident("x")]);
}

#[test]
fn comparison_and_assignment_chars_can_pack_into_identifiers() {
    // The ident regex does not exclude `<`, `>`, or `=`, so adjacent
    // comparison/assignment characters are absorbed into the identifier
    // (the README's `>=` example relies on this).
    assert_eq!(lex("i >= 5"), vec![ident("i"), ident(">="), int(5)]);
    assert_eq!(lex("a < b"), vec![ident("a"), ident("<"), ident("b")]);
    assert_eq!(lex("x==y"), vec![ident("x==y")]);
    assert_eq!(lex("a=b"), vec![ident("a=b")]);
    // `!` IS excluded, so `!=` survives even when packed.
    assert_eq!(lex("a!=b"), vec![ident("a"), Token::Neq, ident("b")]);
}

// ----------------------------------------------------------------
// Operators
// ----------------------------------------------------------------

/// `arithmetic.ax` is a single top-level expression covering `+ - * ** /`.
#[test]
fn lexes_arithmetic_operators() {
    assert_eq!(
        lex(ARITHMETIC),
        vec![
            ident("a"),
            Token::Add,
            ident("b"),
            Token::Sub,
            ident("c"),
            Token::Mul,
            ident("d"),
            Token::Pow,
            ident("e"),
            Token::Div,
            ident("f"),
        ]
    );
}

#[test]
fn lexes_assignment_and_comparison_operators() {
    assert_eq!(
        lex("x = y == z != w"),
        vec![
            ident("x"),
            Token::Assign,
            ident("y"),
            Token::Eq,
            ident("z"),
            Token::Neq,
            ident("w"),
        ]
    );
}

#[test]
fn lexes_range_access_and_method_operators() {
    assert_eq!(
        lex("a .. b ..= c . d : e"),
        vec![
            ident("a"),
            Token::Range,
            ident("b"),
            Token::RangeInclusive,
            ident("c"),
            Token::Access,
            ident("d"),
            Token::Method,
            ident("e"),
        ]
    );
}

#[test]
fn lexes_borrow_operator() {
    assert_eq!(
        lex("&self && x"),
        vec![
            Token::Borrow,
            ident("self"),
            Token::Borrow,
            Token::Borrow,
            ident("x")
        ]
    );
}

#[test]
fn longest_match_wins_between_operators() {
    assert_eq!(lex("a.b"), vec![ident("a"), Token::Access, ident("b")]);
    assert_eq!(lex("a..b"), vec![ident("a"), Token::Range, ident("b")]);
    assert_eq!(
        lex("a..=b"),
        vec![ident("a"), Token::RangeInclusive, ident("b")]
    );
    assert_eq!(lex("a**b"), vec![ident("a"), Token::Pow, ident("b")]);
    assert_eq!(lex("..="), vec![Token::RangeInclusive]);
    assert_eq!(lex("..."), vec![Token::Range, Token::Access]);
}

#[test]
fn minus_before_a_number_or_ident_is_sub() {
    assert_eq!(lex("-5"), vec![Token::Sub, int(5)]);
    assert_eq!(lex("-x"), vec![Token::Sub, ident("x")]);
}

// ----------------------------------------------------------------
// Braces / punctuation
// ----------------------------------------------------------------

#[test]
fn lexes_braces() {
    assert_eq!(
        lex("(){}[]"),
        vec![
            Token::OpenParen,
            Token::CloseParen,
            Token::OpenBlock,
            Token::CloseBlock,
            Token::OpenList,
            Token::CloseList,
        ]
    );
}

#[test]
fn lexes_semicolons_and_item_separators() {
    assert_eq!(
        lex("; , ;"),
        vec![Token::Semicolon, Token::ItemSep, Token::Semicolon]
    );
}

// ----------------------------------------------------------------
// Primitives
// ----------------------------------------------------------------

/// `integer_literal.ax` is a whole file whose only content is `123`.
#[test]
fn lexes_integers() {
    assert_eq!(lex(INTEGER_LITERAL), vec![int(123)]);
    assert_eq!(lex("0 456789"), vec![int(0), int(456789)]);
}

/// `string_literal.ax` is a whole file whose only content is an escaped
/// string literal — a top-level expression that just evals and finishes.
#[test]
fn lexes_double_quoted_strings() {
    assert_eq!(lex(r#""hi""#), vec![string("hi")]);
    assert_eq!(lex(r#""""#), vec![string("")]);
    assert_eq!(lex(r#""a" "b""#), vec![string("a"), string("b")]);
    // Escapes are kept verbatim in the payload, but trim_matches eats
    // both trailing quotes, so a final `\\"` escape loses its quote.
    assert_eq!(lex(STRING_LITERAL), vec![string(r#"say \"hi\"#)]);
}

/// `char_literal.ax` is a whole file whose only content is `'it\'s'`.
#[test]
fn lexes_single_quoted_strings() {
    assert_eq!(lex(r#"'hi'"#), vec![string("hi")]);
    assert_eq!(lex(CHAR_LITERAL), vec![string(r#"it\'s"#)]);
}

/// `boolean_literal.ax` is a whole file whose only content is `true`.
#[test]
fn lexes_bool_literals() {
    assert_eq!(lex(BOOLEAN_LITERAL), vec![Token::Bool(true)]);
    assert_eq!(lex("false"), vec![Token::Bool(false)]);
    assert_eq!(
        lex("true false true"),
        vec![Token::Bool(true), Token::Bool(false), Token::Bool(true)]
    );
}

// ----------------------------------------------------------------
// Comments
// ----------------------------------------------------------------

/// `line_comment.ax` is a whole file whose only content is one comment.
#[test]
fn lexes_line_comments() {
    assert_eq!(lex(LINE_COMMENT), vec![Token::LineComment("hi".into())]);
    assert_eq!(lex("//hi"), vec![Token::LineComment("hi".into())]);
    assert_eq!(
        lex("x // end"),
        vec![ident("x"), Token::LineComment("end".into())]
    );
}

/// `block_comment.ax` is a whole file whose only content is one comment.
#[test]
fn lexes_block_comments() {
    // Note: the current implementation keeps everything after `/*`,
    // trim()-med, so the closing `*/` is part of the payload.
    assert_eq!(
        lex(BLOCK_COMMENT),
        vec![Token::InlineComment("c */".into())]
    );
    assert_eq!(lex("/**/"), vec![Token::InlineComment("*/".into())]);
    assert_eq!(
        lex("/* a\nb */"),
        vec![Token::InlineComment("a\nb */".into())]
    );
}

/// `comments_in_code.ax` shows comments emitted as tokens between code.
#[test]
fn comments_are_emitted_as_tokens_not_skipped() {
    assert_eq!(
        lex(COMMENTS_IN_CODE),
        vec![
            ident("x"),
            Token::InlineComment("mid */".into()),
            ident("y"),
            Token::LineComment("end".into()),
        ]
    );
}

// ----------------------------------------------------------------
// README snippets (verbatim from README.md, via sample_code)
// ----------------------------------------------------------------

/// The full example program from the README, verbatim.
#[test]
fn readme_full_snippet() {
    assert_eq!(
        lex(FULL_PROGRAM),
        vec![
            Token::Function,
            ident("foo"),
            Token::OpenParen,
            ident("arg1"),
            Token::ItemSep,
            ident("arg2"),
            Token::CloseParen,
            Token::OpenBlock,
            Token::Mut,
            ident("a"),
            Token::Assign,
            ident("arg1"),
            Token::Add,
            ident("arg2"),
            Token::Semicolon,
            Token::Mut,
            ident("i"),
            Token::Assign,
            int(0),
            Token::Semicolon,
            Token::Loop,
            Token::OpenBlock,
            Token::If,
            Token::OpenParen,
            ident("i"),
            ident(">="),
            int(5),
            Token::CloseParen,
            Token::OpenBlock,
            Token::Break,
            Token::Semicolon,
            Token::CloseBlock,
            ident("a"),
            Token::Assign,
            ident("a"),
            Token::Add,
            int(1),
            Token::Semicolon,
            ident("i"),
            Token::Assign,
            ident("i"),
            Token::Add,
            int(1),
            Token::Semicolon,
            Token::CloseBlock,
            Token::Return,
            ident("a"),
            Token::Semicolon,
            Token::CloseBlock,
            Token::Let,
            ident("x"),
            Token::Assign,
            int(3),
            Token::Semicolon,
            Token::Let,
            ident("y"),
            Token::Assign,
            ident("foo"),
            Token::OpenParen,
            ident("x"),
            Token::ItemSep,
            int(7),
            Token::CloseParen,
            Token::Semicolon,
            ident("print"),
            Token::OpenParen,
            string("x ="),
            Token::ItemSep,
            ident("x"),
            Token::ItemSep,
            string("(15)"),
            Token::CloseParen,
            Token::Semicolon,
        ]
    );
}

/// The struct/impl language reference block from the README, verbatim (its
/// `<- ...`/`->` annotation lines included, showing how the ident regex
/// absorbs `<`, `>` and `=`).
#[test]
fn readme_snippet_ax_language_reference() {
    assert_eq!(
        lex(LANGUAGE_REFERENCE),
        vec![
            ident("struct"),
            ident("Foo"),
            Token::OpenBlock,
            ident("some_field"),
            Token::Method,
            ident("usize"),
            Token::CloseBlock,
            ident("impl"),
            ident("Foo"),
            Token::OpenBlock,
            Token::Function,
            ident("some_method"),
            Token::OpenParen,
            Token::Borrow,
            ident("self"),
            Token::CloseParen,
            Token::OpenParen,
            ident("a"),
            Token::Method,
            ident("usize"),
            Token::ItemSep,
            ident("b"),
            Token::Method,
            ident("usize"),
            Token::CloseParen,
            Token::Sub,
            ident(">"),
            ident("usize"),
            Token::OpenBlock,
            ident("self"),
            Token::Access,
            ident("some_field"),
            Token::Mul,
            ident("a"),
            Token::Add,
            ident("b"),
            Token::CloseBlock,
            Token::Function,
            ident("some_property"),
            Token::OpenParen,
            Token::Borrow,
            ident("self"),
            Token::CloseParen,
            Token::OpenBlock,
            Token::Return,
            int(4),
            Token::Semicolon,
            Token::CloseBlock,
            Token::CloseBlock,
            Token::Let,
            ident("x"),
            Token::Method,
            ident("Foo"),
            Token::Assign,
            ident("Foo"),
            Token::OpenBlock,
            ident("some_field"),
            Token::Method,
            int(3),
            Token::CloseBlock,
            Token::Semicolon,
            ident("x"),
            Token::Access,
            ident("some_method"),
            ident("<"),
            Token::Sub,
            ident("unbound"),
            ident("function"),
            Token::OpenParen,
            ident("similar"),
            ident("to"),
            ident("Foo"),
            Token::Method,
            Token::Method,
            ident("some_method"),
            Token::CloseParen,
            ident("x"),
            Token::Method,
            ident("some_method"),
            ident("<"),
            Token::Sub,
            ident("bound"),
            ident("method"),
            Token::OpenParen,
            ident("takes"),
            int(2),
            ident("ints"),
            Token::CloseParen,
            ident("x"),
            Token::Method,
            ident("copy"),
        ]
    );
}

/// The let/mut/loop block snippet from the README, verbatim.
#[test]
fn readme_snippet_let_block() {
    assert_eq!(
        lex(LET_LOOP),
        vec![
            Token::Let,
            ident("x"),
            Token::Assign,
            int(5),
            Token::Semicolon,
            Token::Mut,
            ident("x"),
            Token::Assign,
            int(5),
            Token::Semicolon,
            Token::Loop,
            Token::OpenBlock,
            ident("x"),
            Token::Assign,
            ident("x"),
            Token::Add,
            int(1),
            Token::Semicolon,
            Token::CloseBlock,
            Token::Let,
            ident("x"),
            Token::Assign,
            Token::OpenBlock,
            ident("do_some_stuff"),
            Token::OpenParen,
            Token::CloseParen,
            Token::Semicolon,
            int(3),
            Token::CloseBlock,
            Token::Semicolon,
        ]
    );
}

/// Guards the README-derived samples against README edits.
#[test]
fn readme_snippets_stay_in_sync_with_readme() {
    let readme = include_str!("../../README.md");
    assert!(readme.contains(LANGUAGE_REFERENCE));
    assert!(readme.contains(LET_LOOP));
    assert!(readme.contains(FULL_PROGRAM));
}
