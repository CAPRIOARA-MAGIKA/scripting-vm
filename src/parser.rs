use crate::ast::*;
use crate::error::CompileError;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, CompileError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        Ok(stmts)
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, k: TokenType) -> bool {
        self.peek().kind == k
    }

    fn matches(&mut self, k: TokenType) -> bool {
        if self.check(k) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, k: TokenType, msg: &str) -> Result<&Token, CompileError> {
        if self.check(k) {
            Ok(self.advance())
        } else {
            Err(CompileError {
                line: self.peek().line,
                message: msg.into(),
            })
        }
    }

    // ---- declarations ----

    fn declaration(&mut self) -> Result<Stmt, CompileError> {
        if self.matches(TokenType::Var) {
            return self.var_decl();
        }
        if self.matches(TokenType::Fn) {
            return self.fn_decl();
        }
        self.statement()
    }

    fn var_decl(&mut self) -> Result<Stmt, CompileError> {
        let name = self
            .consume(TokenType::Identifier, "expected variable name")?
            .lexeme
            .clone();
        let init = if self.matches(TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(TokenType::Semicolon, "expected ';' after var")?;
        Ok(Stmt::Var { name, init })
    }

    fn fn_decl(&mut self) -> Result<Stmt, CompileError> {
        let name = self
            .consume(TokenType::Identifier, "expected function name")?
            .lexeme
            .clone();
        self.consume(TokenType::LeftParen, "expected '(' after fn name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                params.push(
                    self.consume(TokenType::Identifier, "expected param name")?
                        .lexeme
                        .clone(),
                );
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "expected ')' after params")?;
        self.consume(TokenType::LeftBrace, "expected '{' before fn body")?;
        let body = self.block()?;
        Ok(Stmt::Fn { name, params, body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, CompileError> {
        let mut stmts = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "expected '}'")?;
        Ok(stmts)
    }

    // ---- statements ----

    fn statement(&mut self) -> Result<Stmt, CompileError> {
        if self.matches(TokenType::If) {
            return self.if_stmt();
        }
        if self.matches(TokenType::While) {
            return self.while_stmt();
        }
        if self.matches(TokenType::Return) {
            return self.return_stmt();
        }
        if self.matches(TokenType::Print) {
            return self.print_stmt();
        }
        if self.matches(TokenType::LeftBrace) {
            return Ok(Stmt::Block { stmts: self.block()? });
        }
        self.expr_stmt()
    }

    fn if_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(TokenType::LeftParen, "expected '(' after if")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "expected ')' after if cond")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.matches(TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    fn while_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(TokenType::LeftParen, "expected '(' after while")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "expected ')' after while cond")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { cond, body })
    }

    fn return_stmt(&mut self) -> Result<Stmt, CompileError> {
        let value = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };
        self.consume(TokenType::Semicolon, "expected ';' after return")?;
        Ok(Stmt::Return { value })
    }

    fn print_stmt(&mut self) -> Result<Stmt, CompileError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after print")?;
        Ok(Stmt::Print { expr })
    }

    fn expr_stmt(&mut self) -> Result<Stmt, CompileError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::Expr { expr })
    }

    // ---- expressions (Pratt) ----

    fn expression(&mut self) -> Result<Expr, CompileError> {
        self.parse_precedence(0)
    }

    fn parse_precedence(&mut self, min_prec: u8) -> Result<Expr, CompileError> {
        let mut left = self.primary()?;
        loop {
            let prec = self.current_prec();
            if prec == 0 || min_prec >= prec {
                break;
            }
            // Call: the callee is `left`, the args are parsed inside `finish_call`.
            // Don't pre-parse a right operand here.
            if self.peek().kind == TokenType::LeftParen {
                self.advance(); // consume '('
                left = self.finish_call(left)?;
                continue;
            }
            let op = self.advance().clone();
            // Left-associative: right-hand side recurses at `prec`, not `prec + 1`.
            let right = self.parse_precedence(prec)?;
            left = self.infix(left, op, right)?;
        }
        Ok(left)
    }

    fn current_prec(&self) -> u8 {
        use TokenType::*;
        match self.peek().kind {
            Star | Slash => 12,
            Plus | Minus => 11,
            Greater | GreaterEqual | Less | LessEqual => 9,
            EqualEqual | BangEqual => 8,
            And => 4,
            Or => 3,
            Equal => 1,
            LeftParen => 14,
            _ => 0,
        }
    }

    fn infix(&mut self, left: Expr, op: Token, right: Expr) -> Result<Expr, CompileError> {
        use TokenType::*;
        match op.kind {
            Plus | Minus | Star | Slash | Greater | GreaterEqual | Less | LessEqual
            | EqualEqual | BangEqual => {
                let bop = match op.kind {
                    Plus => BinOp::Add,
                    Minus => BinOp::Sub,
                    Star => BinOp::Mul,
                    Slash => BinOp::Div,
                    EqualEqual => BinOp::Eq,
                    BangEqual => BinOp::Ne,
                    Less => BinOp::Lt,
                    LessEqual => BinOp::Le,
                    Greater => BinOp::Gt,
                    GreaterEqual => BinOp::Ge,
                    _ => unreachable!(),
                };
                Ok(Expr::binary(left, bop, right))
            }
            And | Or => Ok(Expr::Logical {
                op: if op.kind == And { BinOp::And } else { BinOp::Or },
                left: Box::new(left),
                right: Box::new(right),
            }),
            Equal => {
                let name = match left {
                    Expr::Variable { name } => name,
                    _ => {
                        return Err(CompileError {
                            line: op.line,
                            message: "invalid assignment target".into(),
                        })
                    }
                };
                Ok(Expr::Assign {
                    name,
                    value: Box::new(right),
                })
            }
            _ => Err(CompileError {
                line: op.line,
                message: format!("unexpected token in expression: {:?}", op.kind),
            }),
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, CompileError> {
        let mut args = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                args.push(self.parse_precedence(0)?);
                if !self.matches(TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "expected ')' after args")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn primary(&mut self) -> Result<Expr, CompileError> {
        use TokenType::*;
        let t = self.peek().clone();
        match t.kind {
            Number => {
                self.advance();
                let n = match t.literal {
                    crate::token::Literal::Number(n) => n,
                    _ => unreachable!(),
                };
                Ok(Expr::literal(crate::token::Literal::Number(n)))
            }
            String => {
                self.advance();
                let s = match t.literal {
                    crate::token::Literal::Str(s) => crate::token::Literal::Str(s),
                    _ => unreachable!(),
                };
                Ok(Expr::literal(s))
            }
            True => {
                self.advance();
                Ok(Expr::literal(crate::token::Literal::Bool(true)))
            }
            False => {
                self.advance();
                Ok(Expr::literal(crate::token::Literal::Bool(false)))
            }
            Nil => {
                self.advance();
                Ok(Expr::literal(crate::token::Literal::Nil))
            }
            Identifier => {
                self.advance();
                Ok(Expr::Variable { name: t.lexeme.clone() })
            }
            LeftParen => {
                self.advance();
                let e = self.expression()?;
                self.consume(RightParen, "expected ')'")?;
                Ok(Expr::Group { inner: Box::new(e) })
            }
            Minus | Bang => {
                self.advance();
                let op = if t.kind == Minus { UnaryOp::Neg } else { UnaryOp::Not };
                let operand = self.parse_precedence(13)?;
                Ok(Expr::Unary {
                    op,
                    operand: Box::new(operand),
                })
            }
            _ => Err(CompileError {
                line: t.line,
                message: format!("unexpected token: {:?}", t.kind),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::token::Literal;

    fn parse(src: &str) -> Program {
        let toks = Lexer::new(src).scan_tokens().unwrap();
        Parser::new(toks).parse().unwrap()
    }

    #[test]
    fn parses_var_decl() {
        let p = parse("var x = 5;");
        assert_eq!(p.len(), 1);
        match &p[0] {
            Stmt::Var {
                name,
                init: Some(Expr::Literal { value: Literal::Number(n) }),
            } => {
                assert_eq!(name, "x");
                assert_eq!(*n, 5.0);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn parses_fn_decl() {
        let p = parse("fn f(a) { return a; }");
        match &p[0] {
            Stmt::Fn { name, params, body } => {
                assert_eq!(name, "f");
                assert_eq!(params, &vec!["a".to_string()]);
                assert_eq!(body.len(), 1);
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn parses_binary_precedence() {
        let p = parse("1 + 2 * 3;");
        // 1 + (2 * 3)
        match &p[0] {
            Stmt::Expr {
                expr: Expr::Binary { op: BinOp::Add, right, .. },
            } => {
                assert!(matches!(**right, Expr::Binary { op: BinOp::Mul, .. }));
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn parses_logical_and_or() {
        let p = parse("a and b or c;");
        assert!(matches!(
            &p[0],
            Stmt::Expr {
                expr: Expr::Logical { .. }
            }
        ));
    }

    #[test]
    fn parses_call() {
        let p = parse("f(1, 2);");
        match &p[0] {
            Stmt::Expr { expr: Expr::Call { args, .. } } => assert_eq!(args.len(), 2),
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn parses_unary_minus() {
        let p = parse("-5;");
        assert!(matches!(
            &p[0],
            Stmt::Expr {
                expr: Expr::Unary {
                    op: UnaryOp::Neg,
                    ..
                }
            }
        ));
    }
}
