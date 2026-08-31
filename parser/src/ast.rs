#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Size,
    Exponent,
    Cat,
    Eq,
    Neq,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum Node {
    Expression(Expression),
    PlaceExpr(PlaceExpr),
    Statement(Statement),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindKind {
    Borrow,
    BorrowMut,
    Move,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub default: Option<Box<Node>>,
    pub kind: BindKind,
}

#[derive(Clone, Debug)]
pub struct Conditional {
    pub cond: Box<Node>,
    pub eval: Box<Node>,
}

#[derive(Clone, Debug)]
pub enum Statement {
    Continue,
    Break(Option<Box<Node>>),
    Return(Option<Box<Node>>),
    Bind {
        name: String,
        kind: BindKind,
        value: Box<Node>,
    },
    Assign {
        target: PlaceExpr,
        value: Box<Node>,
    },
}

#[derive(Clone, Debug)]
pub enum Expression {
    Integer(i64),
    Bool(bool),
    String(String),
    Err(String),

    Block(Vec<Node>, Box<Node>),
    Conditional(Conditional),
    If(Vec<Conditional>),
    Loop(Box<Node>),

    Operator {
        op: Operator,
        left: Box<Node>,
        right: Box<Node>,
    },
    UnaryOperator {
        op: Operator,
        value: Box<Node>,
    },

    Call(Box<Node>, Vec<Node>),
    Method(Box<Node>, String),
    Fn(Vec<Param>, Box<Node>),
}

#[derive(Clone, Debug)]
pub enum PlaceExpr {
    Identifier(String),
    Access { base: Box<Node>, field: String },
    Index { base: Box<Node>, index: Box<Node> },
}

impl Node {
    pub fn expr(e: Expression) -> Node {
        Node::Expression(e)
    }

    pub fn place(p: PlaceExpr) -> Node {
        Node::PlaceExpr(p)
    }

    pub fn stmt(s: Statement) -> Node {
        Node::Statement(s)
    }

    pub fn binop(op: Operator, l: Node, r: Node) -> Node {
        Node::expr(Expression::Operator {
            op,
            left: Box::new(l),
            right: Box::new(r),
        })
    }

    pub fn unop(op: Operator, v: Node) -> Node {
        Node::expr(Expression::UnaryOperator {
            op,
            value: Box::new(v),
        })
    }

    pub fn call(callee: Node, args: Vec<Node>) -> Node {
        Node::expr(Expression::Call(Box::new(callee), args))
    }

    pub fn cond(cond: Node, eval: Node) -> Node {
        Node::expr(Expression::Conditional(Conditional {
            cond: Box::new(cond),
            eval: Box::new(eval),
        }))
    }

    pub fn assign(target: PlaceExpr, value: Node) -> Node {
        Node::stmt(Statement::Assign {
            target,
            value: Box::new(value),
        })
    }

    pub fn bind(name: impl Into<String>, kind: BindKind, value: Node) -> Node {
        Node::stmt(Statement::Bind {
            name: name.into(),
            kind,
            value: Box::new(value),
        })
    }
}

