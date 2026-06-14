use crate::token::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal { value: Literal },
    Variable { name: String },
    Assign { name: String, value: Box<Expr> },
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Logical { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Unary { op: UnaryOp, operand: Box<Expr> },
    Group { inner: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Empty,
}

impl Expr {
    pub fn literal(v: Literal) -> Self {
        Expr::Literal { value: v }
    }

    pub fn binary(l: Expr, op: BinOp, r: Expr) -> Self {
        Expr::Binary {
            op,
            left: Box::new(l),
            right: Box::new(r),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr { expr: Expr },
    Print { expr: Expr },
    Var { name: String, init: Option<Expr> },
    Block { stmts: Vec<Stmt> },
    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While { cond: Expr, body: Box<Stmt> },
    Return { value: Option<Expr> },
    Fn { name: String, params: Vec<String>, body: Vec<Stmt> },
}

pub type Program = Vec<Stmt>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_expr_in_ast() {
        let e = Expr::binary(
            Expr::literal(Literal::Number(1.0)),
            BinOp::Add,
            Expr::literal(Literal::Number(2.0)),
        );
        if let Expr::Binary { op, .. } = e {
            assert_eq!(op, BinOp::Add);
        } else {
            panic!("not a binary");
        }
    }
}
