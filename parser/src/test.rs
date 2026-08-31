use super::*;
use lexer::Token;
use logos::Logos;

fn lex(src: &str) -> Vec<Token> {
    Token::lexer(src).map(|r| r.unwrap()).collect()
}

fn parse(src: &str) -> Result<Vec<Node>, String> {
    let tokens = lex(src);
    match parser().parse(&tokens[..]).into_result() {
        Ok(nodes) => Ok(nodes),
        Err(errs) => Err(format!("{errs:?}")),
    }
}

/// Every sample program should lex and parse without errors.
#[test]
fn parses_sample_programs() {
    let samples: &[(&str, &str)] = &[
        ("arithmetic.ax", include_str!("../../sample_code/arithmetic.ax")),
        ("block_comment.ax", include_str!("../../sample_code/block_comment.ax")),
        ("boolean_literal.ax", include_str!("../../sample_code/boolean_literal.ax")),
        ("callbacks.ax", include_str!("../../sample_code/callbacks.ax")),
        ("char_literal.ax", include_str!("../../sample_code/char_literal.ax")),
        ("comments_in_code.ax", include_str!("../../sample_code/comments_in_code.ax")),
        ("field_method_namespaces.ax", include_str!("../../sample_code/field_method_namespaces.ax")),
        ("full_program.ax", include_str!("../../sample_code/full_program.ax")),
        ("functions_in_fields.ax", include_str!("../../sample_code/functions_in_fields.ax")),
        ("integer_literal.ax", include_str!("../../sample_code/integer_literal.ax")),
        ("keyword_tour.ax", include_str!("../../sample_code/keyword_tour.ax")),
        ("late_methods.ax", include_str!("../../sample_code/late_methods.ax")),
        ("late_method_signature.ax", include_str!("../../sample_code/late_method_signature.ax")),
        ("let_loop.ax", include_str!("../../sample_code/let_loop.ax")),
        ("line_comment.ax", include_str!("../../sample_code/line_comment.ax")),
        ("methods_call_methods.ax", include_str!("../../sample_code/methods_call_methods.ax")),
        ("object_literal.ax", include_str!("../../sample_code/object_literal.ax")),
        ("objects_from_methods.ax", include_str!("../../sample_code/objects_from_methods.ax")),
        ("string_literal.ax", include_str!("../../sample_code/string_literal.ax")),
    ];

    for (name, src) in samples {
        let result = parse(src);
        assert!(
            matches!(result, Ok(_)),
            "sample {name:?} failed to parse: {result:?}"
        );
    }
}

#[test]
fn parses_arithmetic_with_precedence() {
    // `a + b * c ** d / e` -> Plus(a, Div(Mul(b, Exp(c, d)), e))
    let nodes = parse("a + b * c ** d / e").unwrap();
    assert_eq!(nodes.len(), 1);
    let ok = matches!(&nodes[0],
        Node::Expression(Expression::Operator { op: Operator::Plus, .. })
    );
    assert!(ok, "expected top-level Plus, got {:?}", nodes[0]);
}

#[test]
fn parses_comparisons_as_identifier_operators() {
    // `i >= 5` lexes `>=` as an identifier; it should still become Ge.
    let nodes = parse("i >= 5").unwrap();
    let ok = matches!(&nodes[0],
        Node::Expression(Expression::Operator { op: Operator::Ge, .. })
    );
    assert!(ok, "expected Ge, got {:?}", nodes[0]);
}

#[test]
fn parses_and_or_chains_left_associative() {
    // `a or b and c or d` -> Or(Or(a, And(b, c)), d)
    let nodes = parse("a or b and c or d").unwrap();
    let ok = matches!(&nodes[0],
        Node::Expression(Expression::Operator { op: Operator::Or, left, .. })
            if matches!(left.as_ref(),
                Node::Expression(Expression::Operator { op: Operator::Or, .. })
            )
    );
    assert!(ok, "expected left-assoc Or chain, got {:?}", nodes[0]);
}

#[test]
fn parses_methods_vs_field_access() {
    // `x:value` is a Method, `x.value` is a place Access
    let nodes = parse("x:value").unwrap();
    assert!(matches!(&nodes[0], Node::Expression(Expression::Method(..))));

    let nodes = parse("x.value").unwrap();
    assert!(matches!(&nodes[0], Node::PlaceExpr(PlaceExpr::Access { .. })));
}

#[test]
fn parses_when_with_else_if_chain() {
    let nodes = parse(
        "if (a) { 1 } else if (b) { 2 } else { 3 }",
    )
    .unwrap();
    let ok = matches!(&nodes[0],
        Node::Expression(Expression::If(arms)) if arms.len() == 3
    );
    assert!(ok, "expected 3-armed If, got {:?}", nodes[0]);
}

#[test]
fn parses_for_loops_with_ranges() {
    let nodes = parse("for n in 0..=3 { total = total + n; }").unwrap();
    let ok = matches!(&nodes[0],
        Node::Expression(Expression::For { inclusive: true, .. })
    );
    assert!(ok, "expected inclusive For, got {:?}", nodes[0]);
}

#[test]
fn parses_block_with_statements_and_tail_value() {
    let nodes = parse("{ do_some_stuff(); 3 }").unwrap();
    let ok = matches!(&nodes[0],
        Node::Expression(Expression::Block(items, tail))
            if items.len() == 1 && tail.is_some()
    );
    assert!(ok, "expected block with one statement and a tail, got {:?}", nodes[0]);
}

#[test]
fn parses_dict_vs_block_disambiguation() {
    // `{ x = 3, y = 4 }` is a dict, `{ x; 3 }` is a block
    let nodes = parse("{ x = 3, y = 4 }").unwrap();
    assert!(matches!(&nodes[0], Node::Expression(Expression::Dict(_))));

    let nodes = parse("let p = { x = 3, y = 4 };").unwrap();
    assert!(matches!(&nodes[0], Node::Statement(Statement::Bind { .. })));
}

#[test]
fn parses_late_method_definition() {
    let nodes = parse("counter::add = fn(&self)(n) { self.value + n };").unwrap();
    let ok = matches!(&nodes[0],
        Node::Statement(Statement::Assign { value, .. })
            if matches!(value.as_ref(), Node::Expression(Expression::Fn(..)))
    );
    assert!(ok, "expected assign of an anonymous fn, got {:?}", nodes[0]);
}

#[test]
fn parses_anonymous_fn_with_expression_body() {
    let nodes = parse("fn(n) n * 2").unwrap();
    let ok = matches!(&nodes[0], Node::Expression(Expression::Fn(..)));
    assert!(ok, "expected anonymous fn, got {:?}", nodes[0]);
}

#[test]
fn skips_comments_between_tokens() {
    let nodes = parse("x /* mid */ + // end\n y").unwrap();
    assert_eq!(
        nodes.len(),
        1,
        "expected one expression, got {:?}",
        nodes
    );
    assert!(matches!(&nodes[0], Node::Expression(Expression::Operator { op: Operator::Plus, .. })));
}