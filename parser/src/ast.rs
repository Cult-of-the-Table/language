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
    Gt,
    Lt,
    Ge,
    Le,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum Node {
    Expr(Expr),
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
pub struct KV {
    pub key: String,
    /// Whether the field is a method slot (`:name = fn(...)`).
    pub method: bool,
    pub value: Box<Node>,
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
    Fn(String, Vec<Param>, Box<Node>),
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
pub enum Expr {
    Integer(i64),
    Bool(bool),
    String(String),
    Err(String),

    Block(Vec<Node>, Option<Box<Node>>),
    Dict(Vec<KV>),
    List(Vec<Node>),
    Conditional(Conditional),
    If(Vec<Conditional>),
    Loop(Box<Node>),
    While(Box<Node>, Box<Node>),
    For {
        var: String,
        from: Box<Node>,
        to: Box<Node>,
        inclusive: bool,
        body: Box<Node>,
    },

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
    pub fn expr(e: Expr) -> Node {
        Node::Expr(e)
    }

    pub fn place(p: PlaceExpr) -> Node {
        Node::PlaceExpr(p)
    }

    pub fn stmt(s: Statement) -> Node {
        Node::Statement(s)
    }

    pub fn identifier(name: impl Into<String>) -> Node {
        Node::place(PlaceExpr::Identifier(name.into()))
    }

    pub fn binop(op: Operator, l: Node, r: Node) -> Node {
        Node::expr(Expr::Operator {
            op,
            left: Box::new(l),
            right: Box::new(r),
        })
    }

    pub fn unop(op: Operator, v: Node) -> Node {
        Node::expr(Expr::UnaryOperator {
            op,
            value: Box::new(v),
        })
    }

    pub fn call(callee: Node, args: Vec<Node>) -> Node {
        Node::expr(Expr::Call(Box::new(callee), args))
    }

    pub fn cond(cond: Node, eval: Node) -> Node {
        Node::expr(Expr::Conditional(Conditional {
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
