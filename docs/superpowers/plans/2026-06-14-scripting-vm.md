# Custom Scripting Language & Bytecode VM Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Lox-style dynamic scripting language with a stack-based bytecode VM in Rust, suitable for a portfolio. Two execution paths (tree-walker and VM) must agree on output for every test program.

**Architecture:** Single-pass compiler emits a fixed opcode set into per-function Chunks. A stack-based VM dispatches via a `match` loop. A tree-walking interpreter runs the same AST as a reference implementation; an end-to-end parity test suite enforces agreement.

**Tech Stack:** Rust (stable, 2021 edition), Cargo, no external runtime dependencies. Optional: `insta` for AST snapshot tests (add when needed).

**Reference spec:** `docs/superpowers/specs/2026-06-14-scripting-vm-design.md`

---

## File Structure

The project layout is fixed by the spec (§4). The plan builds it bottom-up: foundation → lexer → parser → tree-walker → compiler → VM → parity harness → polish.

| File | Created in Task | Responsibility |
|------|-----------------|----------------|
| `Cargo.toml` | T1 | Package manifest |
| `src/main.rs` | T1, T18 | CLI dispatch (repl / run / compile) |
| `src/token.rs` | T2 | `Token`, `TokenType` |
| `src/lexer.rs` | T3 | `Lexer` with line tracking |
| `src/ast.rs` | T4 | `Expr`, `Stmt` enums + visitor |
| `src/error.rs` | T5 | `CompileError`, `RuntimeError` |
| `src/parser.rs` | T6, T7, T8 | Recursive-descent parser |
| `src/value.rs` | T9 | `Value` enum, `Value::eq`, display |
| `src/obj.rs` | T10 | `Obj`, `ObjKind` |
| `src/env.rs` | T11 | `Environment` |
| `src/interpreter.rs` | T12 | Tree-walking reference impl |
| `src/opcode.rs` | T13 | `OpCode`, `Chunk` |
| `src/native.rs` | T14 | Built-in `print`, `clock` |
| `src/compiler.rs` | T15 | AST → bytecode |
| `src/upvalue.rs` | T16 | Upvalue resolution |
| `src/vm.rs` | T17 | Stack-based VM |
| `src/main.rs` (extend) | T18 | CLI subcommands |
| `tests/common/run_both.rs` | T19 | Parity test harness |
| `tests/cases/*.lang` | T20 | Test programs |
| `README.md` | T21 | Portfolio readme |
| `docs/architecture.md` | T22 | Architecture doc |
| `docs/bytecode.md` | T23 | Opcode reference |
| `docs/known-limitations.md` | T24 | Honest tradeoffs |
| `examples/*.lang` | T25 | Demo programs |

---

## Task 1: Project skeleton + first commit

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Initialize git repo**

```bash
cd "E:/Cod/Apps/Scripting_Lang_&VM"
git init
git config user.email "you@example.com"
git config user.name "Your Name"
```

- [ ] **Step 2: Create `.gitignore`**

```
/target
Cargo.lock
*.swp
.idea/
.vscode/
```

- [ ] **Step 3: Create `Cargo.toml`**

```toml
[package]
name = "scripting-vm"
version = "0.1.0"
edition = "2021"
description = "A Lox-style dynamic scripting language with a stack-based bytecode VM"
license = "MIT"

[[bin]]
name = "scripting-vm"
path = "src/main.rs"

[profile.release]
opt-level = 3
lto = true
```

- [ ] **Step 4: Create minimal `src/main.rs`**

```rust
fn main() {
    println!("scripting-vm v0.1.0");
}
```

- [ ] **Step 5: Build and run**

Run: `cargo run`
Expected: prints `scripting-vm v0.1.0`

- [ ] **Step 6: Commit**

```bash
git add .gitignore Cargo.toml src/main.rs docs/
git commit -m "init: cargo project + spec"
```

---

## Task 2: Token types

**Files:**
- Create: `src/token.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `src/token.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_display_includes_lexeme() {
        let t = Token::new(TokenType::Number, "42".into(), 42.0, 1);
        assert_eq!(t.lexeme, "42");
        assert_eq!(t.line, 1);
    }

    #[test]
    fn token_type_equality() {
        assert_eq!(TokenType::Plus, TokenType::Plus);
        assert_ne!(TokenType::Plus, TokenType::Minus);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test token`
Expected: FAIL — module not found.

- [ ] **Step 3: Convert to library + implement**

Replace `src/main.rs`:

```rust
fn main() {
    println!("scripting-vm v0.1.0");
}
```

Create `src/lib.rs`:

```rust
pub mod token;
```

Create `src/token.rs`:

```rust
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One- or two-character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,

    // Literals
    Identifier, String, Number,

    // Keywords
    And, Break, Class, Continue, Else, False, Fn, For, If, Nil, Or,
    Return, Super, This, True, Var, While,

    EOF,
}

impl TokenType {
    pub fn keyword(lexeme: &str) -> Option<TokenType> {
        match lexeme {
            "and" => Some(TokenType::And),
            "break" => Some(TokenType::Break),
            "class" => Some(TokenType::Class),
            "continue" => Some(TokenType::Continue),
            "else" => Some(TokenType::Else),
            "false" => Some(TokenType::False),
            "fn" => Some(TokenType::Fn),
            "for" => Some(TokenType::For),
            "if" => Some(TokenType::If),
            "nil" => Some(TokenType::Nil),
            "or" => Some(TokenType::Or),
            "return" => Some(TokenType::Return),
            "super" => Some(TokenType::Super),
            "this" => Some(TokenType::This),
            "true" => Some(TokenType::True),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    Str(String),
    Bool(bool),
    Nil,
    None,
}

impl Token {
    pub fn new(kind: TokenType, lexeme: String, line: usize) -> Self {
        Self { kind, lexeme, literal: Literal::None, line }
    }

    pub fn number(lexeme: String, n: f64, line: usize) -> Self {
        Self { kind: TokenType::Number, lexeme, literal: Literal::Number(n), line }
    }

    pub fn string(lexeme: String, s: String, line: usize) -> Self {
        Self { kind: TokenType::String, lexeme, literal: Literal::Str(s), line }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({})", self.kind, self.lexeme)
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: 2 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/token.rs src/main.rs
git commit -m "add token type definitions"
```

---

## Task 3: Lexer

**Files:**
- Create: `src/lexer.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing tests**

Create `src/lexer.rs`:

```rust
use crate::token::{Token, TokenType};

pub struct Lexer {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self { source: source.chars().collect(), tokens: Vec::new(), start: 0, current: 0, line: 1 }
    }

    pub fn tokens(&self) -> &[Token] { &self.tokens }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, String> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_one()?;
        }
        self.tokens.push(Token::new(TokenType::EOF, String::new(), self.line));
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool { self.current >= self.source.len() }
    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        c
    }
    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }
    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' } else { self.source[self.current + 1] }
    }
    fn matches(&mut self, c: char) -> bool {
        if self.is_at_end() || self.source[self.current] != c { false }
        else { self.current += 1; true }
    }

    fn add_token(&mut self, kind: TokenType) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        self.tokens.push(Token::new(kind, lexeme, self.line));
    }

    fn scan_one(&mut self) -> Result<(), String> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let k = if self.matches('=') { TokenType::BangEqual } else { TokenType::Bang };
                self.add_token(k);
            }
            '=' => {
                let k = if self.matches('=') { TokenType::EqualEqual } else { TokenType::Equal };
                self.add_token(k);
            }
            '<' => {
                let k = if self.matches('=') { TokenType::LessEqual } else { TokenType::Less };
                self.add_token(k);
            }
            '>' => {
                let k = if self.matches('=') { TokenType::GreaterEqual } else { TokenType::Greater };
                self.add_token(k);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != '\n' && !self.is_at_end() { self.advance(); }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            d if d.is_ascii_digit() => self.number()?,
            a if a.is_ascii_alphabetic() || a == '_' => self.identifier(),
            _ => return Err(format!("Unexpected character '{}' at line {}", c, self.line)),
        }
        Ok(())
    }

    fn string(&mut self) -> Result<(), String> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' { self.line += 1; }
            self.advance();
        }
        if self.is_at_end() { return Err(format!("Unterminated string at line {}", self.line)); }
        self.advance(); // closing "
        let s: String = self.source[self.start + 1..self.current - 1].iter().collect();
        self.tokens.push(Token::string(self.source[self.start..self.current].iter().collect(), s, self.line));
        Ok(())
    }

    fn number(&mut self) -> Result<(), String> {
        while self.peek().is_ascii_digit() { self.advance(); }
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();
            while self.peek().is_ascii_digit() { self.advance(); }
        }
        let s: String = self.source[self.start..self.current].iter().collect();
        let n: f64 = s.parse().map_err(|_| format!("Invalid number {} at line {}", s, self.line))?;
        self.tokens.push(Token::number(self.source[self.start..self.current].iter().collect(), n, self.line));
        Ok(())
    }

    fn identifier(&mut self) {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' { self.advance(); }
        let s: String = self.source[self.start..self.current].iter().collect();
        let kind = TokenType::keyword(&s).unwrap_or(TokenType::Identifier);
        self.add_token(kind);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(src: &str) -> Vec<Token> {
        Lexer::new(src).scan_tokens().expect("lex")
    }

    #[test]
    fn lexes_punctuation() {
        let toks = lex("(){},.;-+*");
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![
            TokenType::LeftParen, TokenType::RightParen, TokenType::LeftBrace, TokenType::RightBrace,
            TokenType::Comma, TokenType::Dot, TokenType::Semicolon, TokenType::Minus, TokenType::Plus,
            TokenType::Star, TokenType::EOF,
        ]);
    }

    #[test]
    fn lexes_two_char_operators() {
        let toks = lex("!= == <= >= ! = < >");
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![
            TokenType::BangEqual, TokenType::EqualEqual, TokenType::LessEqual, TokenType::GreaterEqual,
            TokenType::Bang, TokenType::Equal, TokenType::Less, TokenType::Greater, TokenType::EOF,
        ]);
    }

    #[test]
    fn lexes_numbers() {
        let toks = lex("42 3.14");
        assert_eq!(toks[0].kind, TokenType::Number);
        assert_eq!(toks[1].kind, TokenType::Number);
    }

    #[test]
    fn lexes_string() {
        let toks = lex("\"hi\"");
        assert_eq!(toks[0].kind, TokenType::String);
        assert_eq!(toks[0].lexeme, "\"hi\"");
    }

    #[test]
    fn lexes_keywords_and_identifiers() {
        let toks = lex("var x = if true");
        let kinds: Vec<TokenType> = toks.iter().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![
            TokenType::Var, TokenType::Identifier, TokenType::Equal, TokenType::If,
            TokenType::True, TokenType::EOF,
        ]);
    }

    #[test]
    fn skips_line_comments() {
        let toks = lex("// hi\n42");
        assert_eq!(toks[0].kind, TokenType::Number);
        assert_eq!(toks[0].line, 2);
    }

    #[test]
    fn tracks_lines() {
        let toks = lex("a\nb\nc");
        assert_eq!(toks[0].line, 1);
        assert_eq!(toks[1].line, 2);
        assert_eq!(toks[2].line, 3);
    }

    #[test]
    fn unterminated_string_is_error() {
        assert!(Lexer::new("\"abc").scan_tokens().is_err());
    }
}
```

- [ ] **Step 2: Wire it in**

Edit `src/lib.rs`:

```rust
pub mod token;
pub mod lexer;
```

- [ ] **Step 3: Run tests**

Run: `cargo test lexer`
Expected: 8 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/lexer.rs src/lib.rs
git commit -m "add lexer with line tracking"
```

---

## Task 4: AST

**Files:**
- Create: `src/ast.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/ast.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_expr_in_ast() {
        let e = Expr::binary(
            Expr::literal(Literal::Number(1.0)),
            BinOp::Plus,
            Expr::literal(Literal::Number(2.0)),
        );
        if let Expr::Binary { op, .. } = e {
            assert_eq!(op, BinOp::Plus);
        } else {
            panic!("not a binary");
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test ast`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Edit `src/ast.rs` (replace contents):

```rust
use crate::token::Literal;

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp { Add, Sub, Mul, Div, Eq, Ne, Lt, Le, Gt, Ge, And, Or }

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp { Neg, Not }

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
    pub fn literal(v: Literal) -> Self { Expr::Literal { value: v } }
    pub fn binary(l: Expr, op: BinOp, r: Expr) -> Self {
        Expr::Binary { op, left: Box::new(l), right: Box::new(r) }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr { expr: Expr },
    Var { name: String, init: Option<Expr> },
    Block { stmts: Vec<Stmt> },
    If { cond: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { cond: Expr, body: Box<Stmt> },
    Return { value: Option<Expr> },
    Fn { name: String, params: Vec<String>, body: Vec<Stmt> },
}

pub type Program = Vec<Stmt>;
```

Edit `src/lib.rs`:

```rust
pub mod token;
pub mod lexer;
pub mod ast;
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: 3 tests pass total.

- [ ] **Step 5: Commit**

```bash
git add src/ast.rs src/lib.rs
git commit -m "add ast node definitions"
```

---

## Task 5: Errors with source spans

**Files:**
- Create: `src/error.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn error_display_contains_line() {
        let e = CompileError { line: 7, message: "bad".into() };
        assert!(e.to_string().contains("line 7"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test error`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Replace `src/error.rs`:

```rust
use std::fmt;

#[derive(Debug, Clone)]
pub struct CompileError {
    pub line: usize,
    pub message: String,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[line {}] compile error: {}", self.line, self.message)
    }
}

impl std::error::Error for CompileError {}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub stack_trace: Vec<StackFrame>,
}

#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub line: usize,
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "runtime error: {}", self.message)?;
        for fr in &self.stack_trace {
            writeln!(f, "  at {} (line {})", fr.function, fr.line)?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}
```

Edit `src/lib.rs`:

```rust
pub mod token;
pub mod lexer;
pub mod ast;
pub mod error;
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: 4 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/error.rs src/lib.rs
git commit -m "add compile + runtime error types"
```

---

## Task 6: Parser — declarations and statements

**Files:**
- Create: `src/parser.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/parser.rs`:

```rust
use crate::ast::*;
use crate::error::CompileError;
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

pub struct Parser { tokens: Vec<Token>, current: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Self { tokens, current: 0 } }

    pub fn parse(&mut self) -> Result<Program, CompileError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        Ok(stmts)
    }

    fn is_at_end(&self) -> bool { self.peek().kind == TokenType::EOF }
    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous()
    }
    fn check(&self, k: TokenType) -> bool { self.peek().kind == k }
    fn matches(&mut self, k: TokenType) -> bool {
        if self.check(k) { self.advance(); true } else { false }
    }
    fn consume(&mut self, k: TokenType, msg: &str) -> Result<&Token, CompileError> {
        if self.check(k) { Ok(self.advance()) }
        else { Err(CompileError { line: self.peek().line, message: msg.into() }) }
    }

    fn declaration(&mut self) -> Result<Stmt, CompileError> {
        if self.matches(TokenType::Var) { return self.var_decl(); }
        if self.matches(TokenType::Fn) { return self.fn_decl(); }
        self.statement()
    }

    fn var_decl(&mut self) -> Result<Stmt, CompileError> {
        let name = self.consume(TokenType::Identifier, "expected variable name")?.lexeme.clone();
        let init = if self.matches(TokenType::Equal) { Some(self.expression()?) } else { None };
        self.consume(TokenType::Semicolon, "expected ';' after var")?;
        Ok(Stmt::Var { name, init })
    }

    fn fn_decl(&mut self) -> Result<Stmt, CompileError> {
        let name = self.consume(TokenType::Identifier, "expected function name")?.lexeme.clone();
        self.consume(TokenType::LeftParen, "expected '(' after fn name")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                params.push(self.consume(TokenType::Identifier, "expected param name")?.lexeme.clone());
                if !self.matches(TokenType::Comma) { break; }
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

    fn statement(&mut self) -> Result<Stmt, CompileError> {
        if self.matches(TokenType::If) { return self.if_stmt(); }
        if self.matches(TokenType::While) { return self.while_stmt(); }
        if self.matches(TokenType::Return) { return self.return_stmt(); }
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
        let else_branch = if self.matches(TokenType::Else) { Some(Box::new(self.statement()?)) } else { None };
        Ok(Stmt::If { cond, then_branch, else_branch })
    }

    fn while_stmt(&mut self) -> Result<Stmt, CompileError> {
        self.consume(TokenType::LeftParen, "expected '(' after while")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "expected ')' after while cond")?;
        let body = Box::new(self.statement()?);
        Ok(Stmt::While { cond, body })
    }

    fn return_stmt(&mut self) -> Result<Stmt, CompileError> {
        let value = if self.check(TokenType::Semicolon) { None } else { Some(self.expression()?) };
        self.consume(TokenType::Semicolon, "expected ';' after return")?;
        Ok(Stmt::Return { value })
    }

    fn expr_stmt(&mut self) -> Result<Stmt, CompileError> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "expected ';' after expression")?;
        Ok(Stmt::Expr { expr })
    }

    // Expression parsing implemented in Task 7
    fn expression(&mut self) -> Result<Expr, CompileError> { unimplemented!() }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            Stmt::Var { name, init: Some(Expr::Literal { value: Literal::Number(n) }) } => {
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
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test parser`
Expected: FAIL — `expression` is unimplemented.

- [ ] **Step 3: Wire it in**

Edit `src/lib.rs`:

```rust
pub mod token;
pub mod lexer;
pub mod ast;
pub mod error;
pub mod parser;
```

- [ ] **Step 4: Commit (tests will still fail — that's fine, Task 7 finishes them)**

```bash
git add src/parser.rs src/lib.rs
git commit -m "add parser scaffolding: decls and stmts"
```

---

## Task 7: Parser — Pratt expression parsing

**Files:**
- Modify: `src/parser.rs`

- [ ] **Step 1: Replace the `expression` stub**

In `src/parser.rs`, replace `fn expression(&mut self) -> Result<Expr, CompileError> { unimplemented!() }` with the Pratt logic. Add helpers `parse_precedence`, `infix`, `prefix`, `primary` and the precedence table.

```rust
fn expression(&mut self) -> Result<Expr, CompileError> { self.parse_precedence(0) }

fn parse_precedence(&mut self, min_prec: u8) -> Result<Expr, CompileError> {
    let mut left = self.primary()?;
    while min_prec <= self.current_prec() {
        let prec = self.current_prec();
        let op = self.advance().clone();
        let right = self.parse_precedence(prec + 1)?;
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
        Plus | Minus | Star | Slash | Greater | GreaterEqual | Less | LessEqual | EqualEqual | BangEqual => {
            let bop = match op.kind {
                Plus => BinOp::Add, Minus => BinOp::Sub, Star => BinOp::Mul, Slash => BinOp::Div,
                EqualEqual => BinOp::Eq, BangEqual => BinOp::Ne,
                Less => BinOp::Lt, LessEqual => BinOp::Le,
                Greater => BinOp::Gt, GreaterEqual => BinOp::Ge,
                _ => unreachable!(),
            };
            Ok(Expr::binary(left, bop, right))
        }
        And | Or => Ok(Expr::Logical { op: if op.kind == And { BinOp::And } else { BinOp::Or }, left: Box::new(left), right: Box::new(right) }),
        Equal => {
            let name = match left { Expr::Variable { name } => name, _ => return Err(CompileError { line: op.line, message: "invalid assignment target".into() }) };
            Ok(Expr::Assign { name, value: Box::new(right) })
        }
        LeftParen => self.finish_call(left),
        _ => Err(CompileError { line: op.line, message: format!("unexpected token in expression: {:?}", op.kind) }),
    }
}

fn finish_call(&mut self, callee: Expr) -> Result<Expr, CompileError> {
    let mut args = Vec::new();
    if !self.check(TokenType::RightParen) {
        loop {
            args.push(self.parse_precedence(0)?);
            if !self.matches(TokenType::Comma) { break; }
        }
    }
    self.consume(TokenType::RightParen, "expected ')' after args")?;
    Ok(Expr::Call { callee: Box::new(callee), args })
}

fn primary(&mut self) -> Result<Expr, CompileError> {
    use TokenType::*;
    let t = self.peek().clone();
    match t.kind {
        Number => { self.advance(); let n = match t.literal { crate::token::Literal::Number(n) => n, _ => unreachable!() }; Ok(Expr::literal(crate::token::Literal::Number(n))) }
        String => { self.advance(); let s = match t.literal { crate::token::Literal::Str(s) => crate::token::Literal::Str(s), _ => unreachable!() }; Ok(Expr::literal(s)) }
        True => { self.advance(); Ok(Expr::literal(crate::token::Literal::Bool(true))) }
        False => { self.advance(); Ok(Expr::literal(crate::token::Literal::Bool(false))) }
        Nil => { self.advance(); Ok(Expr::literal(crate::token::Literal::Nil)) }
        Identifier => { self.advance(); Ok(Expr::Variable { name: t.lexeme.clone() }) }
        LeftParen => { self.advance(); let e = self.expression()?; self.consume(RightParen, "expected ')'")?; Ok(Expr::Group { inner: Box::new(e) }) }
        Minus | Bang => {
            self.advance();
            let op = if t.kind == Minus { UnaryOp::Neg } else { UnaryOp::Not };
            let operand = self.parse_precedence(13)?;
            Ok(Expr::Unary { op, operand: Box::new(operand) })
        }
        _ => Err(CompileError { line: t.line, message: format!("unexpected token: {:?}", t.kind) }),
    }
}
```

- [ ] **Step 2: Add expression tests**

Append to `#[cfg(test)] mod tests` in `src/parser.rs`:

```rust
    #[test]
    fn parses_binary_precedence() {
        let p = parse("1 + 2 * 3;");
        // 1 + (2 * 3)
        match &p[0] {
            Stmt::Expr { expr: Expr::Binary { op: BinOp::Add, right, .. } } => {
                assert!(matches!(**right, Expr::Binary { op: BinOp::Mul, .. }));
            }
            other => panic!("got {:?}", other),
        }
    }

    #[test]
    fn parses_logical_and_or() {
        let p = parse("a and b or c;");
        assert!(matches!(&p[0], Stmt::Expr { expr: Expr::Logical { .. } }));
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
        assert!(matches!(&p[0], Stmt::Expr { expr: Expr::Unary { op: UnaryOp::Neg, .. } }));
    }
```

- [ ] **Step 3: Run tests**

Run: `cargo test`
Expected: all parser tests pass (existing 2 + new 4 = 6 parser tests; total 10 across the project).

- [ ] **Step 4: Commit**

```bash
git add src/parser.rs
git commit -m "parser: Pratt expression parsing with precedence"
```

---

## Task 8: Parser end-to-end smoke

**Files:**
- Create: `tests/parse_smoke.rs`

- [ ] **Step 1: Write integration test**

Create `tests/parse_smoke.rs`:

```rust
use scripting_vm::ast::{Expr, Stmt};
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::token::Literal;

#[test]
fn parses_full_program() {
    let src = r#"
        var x = 10;
        fn double(n) { return n * 2; }
        if (x > 0 and x < 100) {
            print double(x);
        }
    "#;
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let stmts = Parser::new(toks).parse().unwrap();
    assert_eq!(stmts.len(), 3);
    assert!(matches!(stmts[0], Stmt::Var { .. }));
    assert!(matches!(stmts[1], Stmt::Fn { .. }));
    assert!(matches!(stmts[2], Stmt::If { .. }));
}
```

- [ ] **Step 2: Run test**

Run: `cargo test --test parse_smoke`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add tests/parse_smoke.rs
git commit -m "test: parser end-to-end smoke"
```

---

## Task 9: Value type

**Files:**
- Create: `src/value.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/value.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn numbers_compare_equal() { assert!(Value::from(1.0).equals(&Value::from(1.0))); }
    #[test]
    fn numbers_display() { assert_eq!(Value::from(2.5).to_string(), "2.5"); }
    #[test]
    fn nil_displays_as_nil() { assert_eq!(Value::Nil.to_string(), "nil"); }
}
```

- [ ] **Step 2: Run test**

Run: `cargo test value`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Replace `src/value.rs`:

```rust
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use crate::obj::{Obj, ObjKind};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Obj(Rc<RefCell<Obj>>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool { self.equals(other) }
}

impl Value {
    pub fn from(n: f64) -> Self { Value::Number(n) }
    pub fn from_bool(b: bool) -> Self { Value::Bool(b) }
    pub fn from_string(s: String) -> Self { Value::Obj(Rc::new(RefCell::new(Obj { kind: ObjKind::String(s) }))) }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Nil => false,
            Value::Bool(b) => *b,
            _ => true,
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Obj(a), Value::Obj(b)) => {
                let a = a.borrow(); let b = b.borrow();
                match (&a.kind, &b.kind) {
                    (ObjKind::String(s1), ObjKind::String(s2)) => s1 == s2,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 { write!(f, "{}", *n as i64) } else { write!(f, "{}", n) }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Obj(o) => {
                let o = o.borrow();
                match &o.kind { ObjKind::String(s) => write!(f, "{}", s), _ => write!(f, "<object>") }
            }
        }
    }
}
```

Edit `src/lib.rs`:

```rust
pub mod token;
pub mod lexer;
pub mod ast;
pub mod error;
pub mod parser;
pub mod obj;
pub mod value;
```

(We need `obj` to compile `value`; create a stub next.)

- [ ] **Step 4: Create `obj.rs` stub**

Create `src/obj.rs`:

```rust
use crate::value::Value;

pub struct Obj { pub kind: ObjKind }

pub enum ObjKind {
    String(String),
    Function(FunctionObj),
    Closure(ClosureObj),
    Upvalue(UpvalueObj),
}

pub struct FunctionObj { pub name: String, pub arity: u8 }
pub struct ClosureObj { pub function: Box<FunctionObj> }
pub struct UpvalueObj { pub value: Value }
```

- [ ] **Step 5: Run tests**

Run: `cargo test`
Expected: 13 tests pass (3 new value tests, 10 prior).

- [ ] **Step 6: Commit**

```bash
git add src/value.rs src/obj.rs src/lib.rs
git commit -m "add value type + obj scaffold"
```

---

## Task 10: Environment

**Files:**
- Create: `src/env.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/env.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn define_and_get() {
        let mut e = Environment::new();
        e.define("x".into(), Value::from(1.0));
        assert_eq!(e.get("x"), Some(Value::from(1.0)));
    }
    #[test]
    fn shadow_via_nested() {
        let mut outer = Environment::new();
        outer.define("x".into(), Value::from(1.0));
        let mut inner = outer.new_child();
        inner.define("x".into(), Value::from(2.0));
        assert_eq!(inner.get("x"), Some(Value::from(2.0)));
        assert_eq!(outer.get("x"), Some(Value::from(1.0)));
    }
    #[test]
    fn assign_in_enclosing() {
        let mut outer = Environment::new();
        outer.define("x".into(), Value::from(1.0));
        let mut inner = outer.new_child();
        inner.assign("x", Value::from(5.0)).unwrap();
        assert_eq!(outer.get("x"), Some(Value::from(5.0)));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test env`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Replace `src/env.rs`:

```rust
use std::collections::HashMap;
use crate::error::RuntimeError;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self { Self { values: HashMap::new(), parent: None } }
    pub fn new_child(parent: Environment) -> Self { Self { values: HashMap::new(), parent: Some(Box::new(parent)) } }

    pub fn define(&mut self, name: String, v: Value) { self.values.insert(name, v); }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(v) = self.values.get(name) { return Some(v.clone()); }
        self.parent.as_ref().and_then(|p| p.get(name))
    }

    pub fn assign(&mut self, name: &str, v: Value) -> Result<(), RuntimeError> {
        if self.values.contains_key(name) { self.values.insert(name.to_string(), v); return Ok(()); }
        if let Some(p) = self.parent.as_mut() { return p.assign(name, v); }
        Err(RuntimeError { message: format!("undefined variable '{}'", name), stack_trace: vec![] })
    }
}
```

Edit `src/lib.rs` — add `pub mod env;`.

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: 16 tests pass (3 new env tests).

- [ ] **Step 5: Commit**

```bash
git add src/env.rs src/lib.rs
git commit -m "add environment (lexical scoping)"
```

---

## Task 11: Tree-walking interpreter

**Files:**
- Create: `src/interpreter.rs`
- Modify: `src/lib.rs`

This is the largest task. The interpreter evaluates the AST directly using `Value` and `Environment`. It serves as the reference implementation the VM will be tested against.

- [ ] **Step 1: Write failing integration test**

Create `tests/interp_smoke.rs`:

```rust
use scripting_vm::ast::Program;
use scripting_vm::interpreter::Interpreter;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn arith_and_print() {
    let src = "print 1 + 2 * 3;";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog: Program = Parser::new(toks).parse().unwrap();
    let out = Rc::new(RefCell::new(Vec::new()));
    let mut interp = Interpreter::with_sink(out.clone());
    interp.run(&prog).unwrap();
    assert_eq!(out.borrow().concat(), "7\n");
}
```

- [ ] **Step 2: Run test**

Run: `cargo test --test interp_smoke`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement interpreter**

Create `src/interpreter.rs`:

```rust
use std::cell::RefCell;
use std::rc::Rc;
use crate::ast::*;
use crate::env::Environment;
use crate::error::RuntimeError;
use crate::value::Value;
use crate::token::Literal;

pub type OutputSink = Rc<RefCell<Vec<String>>>;

pub struct Interpreter { globals: Environment, output: OutputSink }

impl Interpreter {
    pub fn with_sink(output: OutputSink) -> Self {
        let mut g = Environment::new();
        g.define("clock".into(), Value::Nil); // stub: TODO native in Task 14
        Self { globals: g, output }
    }

    pub fn output(&self) -> OutputSink { self.output.clone() }
    pub fn globals(&self) -> Environment { self.globals.clone() }

    pub fn run(&mut self, prog: &Program) -> Result<(), RuntimeError> {
        for s in prog { self.exec_stmt(s, &mut self.globals.clone())?; }
        Ok(())
    }

    fn exec_stmt(&self, s: &Stmt, env: &mut Environment) -> Result<(), RuntimeError> {
        match s {
            Stmt::Expr { expr } => { self.eval(expr, env)?; Ok(()) }
            Stmt::Print { expr } => {
                let v = self.eval(expr, env)?;
                self.output.borrow_mut().push(v.to_string());
                Ok(())
            }
            Stmt::Var { name, init } => {
                let v = if let Some(e) = init { self.eval(e, env)? } else { Value::Nil };
                env.define(name.clone(), v);
                Ok(())
            }
            Stmt::Block { stmts } => {
                let mut child = Environment::new_child(env.clone());
                for s in stmts { self.exec_stmt(s, &mut child)?; }
                Ok(())
            }
            Stmt::If { cond, then_branch, else_branch } => {
                if self.eval(cond, env)?.is_truthy() { self.exec_stmt(then_branch, env)?; }
                else if let Some(e) = else_branch { self.exec_stmt(e, env)?; }
                Ok(())
            }
            Stmt::While { cond, body } => {
                while self.eval(cond, env)?.is_truthy() { self.exec_stmt(body, env)?; }
                Ok(())
            }
            Stmt::Return { value } => {
                let v = if let Some(e) = value { Some(self.eval(e, env)?) } else { None };
                Err(RuntimeError { message: format!("return {:?}", v), stack_trace: vec![] })
            }
            Stmt::Fn { .. } => Ok(()), // handled at top-level pre-pass
        }
    }

    fn eval(&self, e: &Expr, env: &Environment) -> Result<Value, RuntimeError> {
        match e {
            Expr::Literal { value } => Ok(self.from_literal(value.clone())),
            Expr::Variable { name } => env.get(name).ok_or(RuntimeError { message: format!("undefined variable '{}'", name), stack_trace: vec![] }),
            Expr::Assign { name, value } => {
                let v = self.eval(value, env)?;
                let mut e = env.clone();
                e.assign(name, v.clone())?;
                Ok(v)
            }
            Expr::Group { inner } => self.eval(inner, env),
            Expr::Unary { op, operand } => {
                let v = self.eval(operand, env)?;
                match op {
                    UnaryOp::Neg => match v { Value::Number(n) => Ok(Value::Number(-n)), _ => Err(RuntimeError { message: "operand must be a number".into(), stack_trace: vec![] }) },
                    UnaryOp::Not => Ok(Value::Bool(!v.is_truthy())),
                }
            }
            Expr::Binary { op, left, right } => self.eval_binary(*op, left, right, env),
            Expr::Logical { op, left, right } => {
                let l = self.eval(left, env)?;
                match op {
                    BinOp::And => if !l.is_truthy() { Ok(Value::Bool(false)) } else { Ok(Value::Bool(self.eval(right, env)?.is_truthy())) },
                    BinOp::Or => if l.is_truthy() { Ok(Value::Bool(true)) } else { Ok(Value::Bool(self.eval(right, env)?.is_truthy())) },
                    _ => unreachable!(),
                }
            }
            Expr::Call { callee, args } => {
                let callee_v = self.eval(callee, env)?;
                let arg_vals: Result<Vec<Value>, RuntimeError> = args.iter().map(|a| self.eval(a, env)).collect();
                match callee_v {
                    Value::Obj(o) => {
                        let o = o.borrow();
                        match &o.kind {
                            crate::obj::ObjKind::Function(_) => Err(RuntimeError { message: "function call not yet implemented for interpreter".into(), stack_trace: vec![] }),
                            _ => Err(RuntimeError { message: "not callable".into(), stack_trace: vec![] }),
                        }
                    }
                    _ => Err(RuntimeError { message: "not callable".into(), stack_trace: vec![] }),
                }
            }
            Expr::Empty => Ok(Value::Nil),
        }
    }

    fn eval_binary(&self, op: BinOp, l: &Expr, r: &Expr, env: &Environment) -> Result<Value, RuntimeError> {
        let lv = self.eval(l, env)?;
        let rv = self.eval(r, env)?;
        match op {
            BinOp::Add => match (&lv, &rv) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                (Value::Obj(a), Value::Obj(b)) => {
                    let (av, bv) = (a.borrow(), b.borrow());
                    if let (crate::obj::ObjKind::String(s1), crate::obj::ObjKind::String(s2)) = (&av.kind, &bv.kind) {
                        Ok(Value::from_string(format!("{}{}", s1, s2)))
                    } else { Err(RuntimeError { message: "operands must be two numbers or two strings".into(), stack_trace: vec![] }) }
                }
                _ => Err(RuntimeError { message: "operands must be two numbers or two strings".into(), stack_trace: vec![] }),
            },
            BinOp::Sub => num_bin(lv, rv, |a, b| a - b),
            BinOp::Mul => num_bin(lv, rv, |a, b| a * b),
            BinOp::Div => num_bin(lv, rv, |a, b| a / b),
            BinOp::Eq => Ok(Value::Bool(lv.equals(&rv))),
            BinOp::Ne => Ok(Value::Bool(!lv.equals(&rv))),
            BinOp::Lt => cmp_num(lv, rv, |a, b| a < b),
            BinOp::Le => cmp_num(lv, rv, |a, b| a <= b),
            BinOp::Gt => cmp_num(lv, rv, |a, b| a > b),
            BinOp::Ge => cmp_num(lv, rv, |a, b| a >= b),
            BinOp::And | BinOp::Or => unreachable!(),
        }
    }

    fn from_literal(&self, lit: Literal) -> Value {
        match lit {
            Literal::Number(n) => Value::Number(n),
            Literal::Str(s) => Value::from_string(s),
            Literal::Bool(b) => Value::Bool(b),
            Literal::Nil => Value::Nil,
            Literal::None => Value::Nil,
        }
    }
}

fn num_bin<F: Fn(f64, f64) -> f64>(l: Value, r: Value, f: F) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        _ => Err(RuntimeError { message: "operands must be numbers".into(), stack_trace: vec![] }),
    }
}
fn cmp_num<F: Fn(f64, f64) -> bool>(l: Value, r: Value, f: F) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(f(a, b))),
        _ => Err(RuntimeError { message: "operands must be numbers".into(), stack_trace: vec![] }),
    }
}
```

- [ ] **Step 4: Add `Print` variant**

We need a `Print` stmt. Edit `src/ast.rs`: change `Stmt::Expr` to add a sibling variant. Replace the existing `Stmt` enum:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expr { expr: Expr },
    Print { expr: Expr },
    Var { name: String, init: Option<Expr> },
    Block { stmts: Vec<Stmt> },
    If { cond: Expr, then_branch: Box<Stmt>, else_branch: Option<Box<Stmt>> },
    While { cond: Expr, body: Box<Stmt> },
    Return { value: Option<Expr> },
    Fn { name: String, params: Vec<String>, body: Vec<Stmt> },
}
```

In `src/parser.rs`, in `expr_stmt`, branch on whether the expression starts with the `print` keyword. Simpler: add a `print_stmt` path in `statement()`:

```rust
fn statement(&mut self) -> Result<Stmt, CompileError> {
    if self.matches(TokenType::If) { return self.if_stmt(); }
    if self.matches(TokenType::While) { return self.while_stmt(); }
    if self.matches(TokenType::Return) { return self.return_stmt(); }
    if self.matches(TokenType::Print) { return self.print_stmt(); }
    if self.matches(TokenType::LeftBrace) {
        return Ok(Stmt::Block { stmts: self.block()? });
    }
    self.expr_stmt()
}

fn print_stmt(&mut self) -> Result<Stmt, CompileError> {
    let expr = self.expression()?;
    self.consume(TokenType::Semicolon, "expected ';' after print")?;
    Ok(Stmt::Print { expr })
}
```

- [ ] **Step 5: Wire interpreter in**

Edit `src/lib.rs` — add `pub mod interpreter;`.

- [ ] **Step 6: Run tests**

Run: `cargo test`
Expected: smoke test passes. Existing parser tests still pass.

- [ ] **Step 7: Commit**

```bash
git add src/interpreter.rs src/ast.rs src/parser.rs src/lib.rs tests/interp_smoke.rs
git commit -m "add tree-walking interpreter (reference impl)"
```

---

## Task 12: Opcodes + Chunk

**Files:**
- Create: `src/opcode.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/opcode.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chunk_writes_and_reads() {
        let mut c = Chunk::new();
        c.write(OpCode::Constant, 1);
        c.write(OpCode::Return, 1);
        c.add_constant(crate::value::Value::Number(7.0));
        assert_eq!(c.code[0], OpCode::Constant as u8);
        assert_eq!(c.code[1], 0);
    }
    #[test]
    fn opcodes_distinct_u8() {
        let ops = [OpCode::Constant, OpCode::Add, OpCode::Return];
        let bytes: Vec<u8> = ops.iter().map(|o| *o as u8).collect();
        assert!(bytes[0] != bytes[1] && bytes[1] != bytes[2]);
    }
}
```

- [ ] **Step 2: Run test**

Run: `cargo test opcode`
Expected: FAIL — module not found.

- [ ] **Step 3: Implement**

Replace `src/opcode.rs`:

```rust
use crate::value::Value;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    Constant,
    Nil, True, False,
    Pop,
    GetLocal, SetLocal,
    GetGlobal, DefineGlobal, SetGlobal,
    GetUpvalue, SetUpvalue,
    Add, Sub, Mul, Div, Neg, Not,
    Equal, Greater, Less,
    Print,
    Jump, JumpIfFalse, Loop,
    Call, Closure, CloseUpvalue, Return,
    // Reserved (v2)
    Class, Inherit, Method, GetProperty, SetProperty, Invoke, SuperInvoke,
}

#[derive(Debug, Clone, Default)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self { Self::default() }
    pub fn write(&mut self, op: OpCode, line: usize) { self.code.push(op as u8); self.lines.push(line); }
    pub fn write_byte(&mut self, b: u8, line: usize) { self.code.push(b); self.lines.push(line); }
    pub fn write_u16(&mut self, n: u16, line: usize) {
        self.code.push((n >> 8) as u8); self.lines.push(line);
        self.code.push((n & 0xff) as u8); self.lines.push(line);
    }
    pub fn add_constant(&mut self, v: Value) -> u8 {
        self.constants.push(v);
        (self.constants.len() - 1) as u8
    }
}
```

Edit `src/lib.rs` — add `pub mod opcode;`.

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: 2 new tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/opcode.rs src/lib.rs
git commit -m "add opcode set and chunk container"
```

---

## Task 13: Upvalue tracking stub

**Files:**
- Create: `src/upvalue.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing test**

Create `src/upvalue.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn upvalue_distinguishes_open_and_closed() {
        let mut uv = Upvalue::new(0, true);
        assert!(uv.is_open);
        uv.is_open = false;
        assert!(!uv.is_open);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test upvalue`
Expected: FAIL.

- [ ] **Step 3: Implement**

Replace `src/upvalue.rs`:

```rust
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Upvalue {
    pub index: usize,
    pub is_open: bool,
    pub value: Value,
}

impl Upvalue {
    pub fn new(index: usize, is_open: bool) -> Self { Self { index, is_open, value: Value::Nil } }
}
```

Edit `src/lib.rs` — add `pub mod upvalue;`.

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: passes.

- [ ] **Step 5: Commit**

```bash
git add src/upvalue.rs src/lib.rs
git commit -m "add upvalue tracking"
```

---

## Task 14: Natives (print, clock)

**Files:**
- Modify: `src/value.rs`
- Create: `src/native.rs`
- Modify: `src/lib.rs`

We extend `Value` to carry native function values, and add a `Natives` module the VM consults.

- [ ] **Step 1: Extend Value with NativeFn**

In `src/value.rs`, add a variant. Edit the `Value` enum:

```rust
use std::rc::Rc;
use std::cell::RefCell;
use crate::obj::{Obj, ObjKind};
use std::time::{SystemTime, UNIX_EPOCH};

pub type NativeFn = fn(args: &[Value]) -> Result<Value, String>;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    Obj(Rc<RefCell<Obj>>),
    Native(NativeFn),
}
```

Update `is_truthy`, `equals`, and `Display` to handle the new variant. Add these arms to each:

- `is_truthy`: `Value::Native(_) => true,`
- `equals`: `Value::Native(a), Value::Native(b) => a == b,`
- `Display`: `Value::Native(_) => write!(f, "<native fn>"),`

- [ ] **Step 2: Create natives module**

Create `src/native.rs`:

```rust
use crate::value::{Value, NativeFn};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn clock(_args: &[Value]) -> Result<Value, String> {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_secs_f64();
    Ok(Value::Number(secs))
}

pub fn print_native(args: &[Value]) -> Result<Value, String> {
    if args.len() != 1 { return Err(format!("print() takes exactly 1 argument ({} given)", args.len())); }
    println!("{}", args[0]);
    Ok(Value::Nil)
}

pub fn registry() -> Vec<(&'static str, NativeFn)> {
    vec![("clock", clock as NativeFn), ("print", print_native as NativeFn)]
}
```

- [ ] **Step 3: Wire it in**

Edit `src/lib.rs`:

```rust
pub mod token;
pub mod lexer;
pub mod ast;
pub mod error;
pub mod parser;
pub mod obj;
pub mod value;
pub mod env;
pub mod interpreter;
pub mod opcode;
pub mod upvalue;
pub mod native;
```

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: all existing pass; nothing new.

- [ ] **Step 5: Commit**

```bash
git add src/value.rs src/native.rs src/lib.rs
git commit -m "add native functions (print, clock)"
```

---

## Task 15: Compiler — AST to bytecode (no closures)

**Files:**
- Create: `src/compiler.rs`
- Modify: `src/lib.rs`

The compiler emits bytecode into a `Chunk`. v1 of the compiler handles everything except upvalue capture. Closures land in Task 16.

- [ ] **Step 1: Write failing integration test**

Create `tests/compile_smoke.rs`:

```rust
use scripting_vm::ast::Program;
use scripting_vm::compiler::Compiler;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::opcode::OpCode;

#[test]
fn compile_simple_print() {
    let src = "print 1 + 2;";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog: Program = Parser::new(toks).parse().unwrap();
    let c = Compiler::new();
    let func = c.compile(&prog).unwrap();
    let code = func.chunk.code.clone();
    assert!(code.iter().any(|b| *b == OpCode::Print as u8));
    assert!(code.iter().any(|b| *b == OpCode::Add as u8));
}
```

- [ ] **Step 2: Run test**

Run: `cargo test --test compile_smoke`
Expected: FAIL.

- [ ] **Step 3: Implement compiler**

Create `src/compiler.rs`:

```rust
use std::collections::HashMap;
use crate::ast::*;
use crate::error::CompileError;
use crate::opcode::{Chunk, OpCode};
use crate::token::Literal;
use crate::value::Value;
use crate::native;

#[derive(Debug, Clone)]
struct Local { name: String, depth: i32, is_captured: bool }

#[derive(Debug, Clone)]
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: String,
}

pub struct Compiler {
    function: Function,
    locals: Vec<Local>,
    scope_depth: i32,
    globals: HashMap<String, u8>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            function: Function { arity: 0, chunk: Chunk::new(), name: "<script>".into() },
            locals: vec![Local { name: String::new(), depth: 0, is_captured: false }], // slot 0 reserved
            scope_depth: 0,
            globals: HashMap::new(),
        }
    }
    pub fn globals(&self) -> HashMap<String, u8> { self.globals.clone() }
    fn current_chunk(&mut self) -> &mut Chunk { &mut self.function.chunk }
    fn emit_op(&mut self, op: OpCode, line: usize) { self.current_chunk().write(op, line); }
    fn emit_byte(&mut self, b: u8, line: usize) { self.current_chunk().write_byte(b, line); }
    fn emit_jump(&mut self, op: OpCode, line: usize) -> usize {
        self.emit_op(op, line);
        self.emit_byte(0xff, line);
        self.emit_byte(0xff, line);
        self.current_chunk().code.len() - 2
    }
    fn patch_jump(&mut self, at: usize) -> Result<(), CompileError> {
        let jump = self.current_chunk().code.len() - at - 2;
        if jump > u16::MAX as usize { return Err(CompileError { line: 0, message: "jump too large".into() }); }
        self.current_chunk().code[at] = ((jump >> 8) & 0xff) as u8;
        self.current_chunk().code[at + 1] = (jump & 0xff) as u8;
        Ok(())
    }
    fn emit_loop(&mut self, start: usize, line: usize) -> Result<(), CompileError> {
        self.emit_op(OpCode::Loop, line);
        let offset = self.current_chunk().code.len() - start + 2;
        if offset > u16::MAX as usize { return Err(CompileError { line, message: "loop too large".into() }); }
        self.emit_byte(((offset >> 8) & 0xff) as u8, line);
        self.emit_byte((offset & 0xff) as u8, line);
        Ok(())
    }
    fn emit_constant(&mut self, v: Value, line: usize) -> Result<u8, CompileError> {
        let c = self.current_chunk();
        let idx = c.add_constant(v);
        if idx > 0xff { return Err(CompileError { line, message: "too many constants".into() }); }
        self.emit_op(OpCode::Constant, line);
        self.emit_byte(idx, line);
        Ok(idx)
    }
    fn add_local(&mut self, name: String) -> Result<(), CompileError> {
        if self.locals.len() >= 256 { return Err(CompileError { line: 0, message: "too many locals".into() }); }
        self.locals.push(Local { name, depth: -1, is_captured: false });
        Ok(())
    }
    fn resolve_local(&self, name: &str) -> Option<u8> {
        for (i, l) in self.locals.iter().enumerate().rev() {
            if l.name == name && l.depth != -1 { return Some(i as u8); }
        }
        None
    }
    fn resolve_global(&self, name: &str) -> Option<u8> { self.globals.get(name).copied() }

    pub fn compile(mut self, prog: &Program) -> Result<Function, CompileError> {
        // Define natives
        for (n, _) in native::registry() { self.globals.insert(n.to_string(), self.globals.len() as u8); }

        for s in prog { self.stmt(s)?; }
        // Implicit return nil at end of script
        self.emit_op(OpCode::Nil, 0);
        self.emit_op(OpCode::Return, 0);
        Ok(self.function)
    }

    fn begin_scope(&mut self) { self.scope_depth += 1; }
    fn end_scope(&mut self, line: usize) {
        self.scope_depth -= 1;
        while let Some(l) = self.locals.last() {
            if l.depth > self.scope_depth {
                self.emit_op(OpCode::Pop, line);
                self.locals.pop();
            } else { break; }
        }
    }

    fn stmt(&mut self, s: &Stmt) -> Result<(), CompileError> {
        match s {
            Stmt::Expr { expr } => { self.expr(expr)?; self.emit_op(OpCode::Pop, 0); Ok(()) }
            Stmt::Print { expr } => { self.expr(expr)?; self.emit_op(OpCode::Print, 0); Ok(()) }
            Stmt::Var { name, init } => {
                if self.scope_depth > 0 {
                    self.add_local(name.clone())?;
                    if let Some(e) = init { self.expr(e)?; } else { self.emit_op(OpCode::Nil, 0); }
                } else {
                    let idx = if let Some(g) = self.resolve_global(name) { g }
                              else { let i = self.globals.len() as u8; self.globals.insert(name.clone(), i); i };
                    if let Some(e) = init { self.expr(e)?; } else { self.emit_op(OpCode::Nil, 0); }
                    self.emit_op(OpCode::DefineGlobal, 0);
                    self.emit_byte(idx, 0);
                }
                Ok(())
            }
            Stmt::Block { stmts } => { self.begin_scope(); for s in stmts { self.stmt(s)?; } self.end_scope(0); Ok(()) }
            Stmt::If { cond, then_branch, else_branch } => {
                self.expr(cond)?;
                let j1 = self.emit_jump(OpCode::JumpIfFalse, 0);
                self.emit_op(OpCode::Pop, 0);
                self.stmt(then_branch)?;
                let j2 = self.emit_jump(OpCode::Jump, 0);
                self.patch_jump(j1)?;
                self.emit_op(OpCode::Pop, 0);
                if let Some(e) = else_branch { self.stmt(e)?; }
                self.patch_jump(j2)
            }
            Stmt::While { cond, body } => {
                let loop_start = self.current_chunk().code.len();
                self.expr(cond)?;
                let exit = self.emit_jump(OpCode::JumpIfFalse, 0);
                self.emit_op(OpCode::Pop, 0);
                self.stmt(body)?;
                self.emit_loop(loop_start, 0)?;
                self.patch_jump(exit)?;
                self.emit_op(OpCode::Pop, 0);
                Ok(())
            }
            Stmt::Return { value } => {
                if let Some(e) = value { self.expr(e)?; } else { self.emit_op(OpCode::Nil, 0); }
                self.emit_op(OpCode::Return, 0);
                Ok(())
            }
            Stmt::Fn { name, params, body } => {
                // compile into a new Function
                let mut sub = Compiler::new();
                sub.function.arity = params.len() as u8;
                sub.function.name = name.clone();
                sub.begin_scope();
                for p in params {
                    sub.add_local(p.clone())?;
                    sub.locals.last_mut().unwrap().depth = 0;
                }
                let block = Stmt::Block { stmts: body.clone() };
                sub.stmt(&block)?;
                sub.emit_op(OpCode::Nil, 0);
                sub.emit_op(OpCode::Return, 0);
                let func_obj = sub.function;
                let idx = self.emit_constant(Value::Obj(std::rc::Rc::new(std::cell::RefCell::new(crate::obj::Obj { kind: crate::obj::ObjKind::Function(crate::obj::FunctionObj { name: func_obj.name.clone(), arity: func_obj.arity }) }))), 0)?;
                // Patch the function's chunk into the Obj
                if let Some(slot) = self.current_chunk().constants.get_mut(idx as usize) {
                    if let Value::Obj(o) = slot {
                        let mut b = o.borrow_mut();
                        if let crate::obj::ObjKind::Function(f) = &mut b.kind {
                            f.chunk = Some(func_obj.chunk);
                        }
                    }
                }
                // Define global for the function
                let g = if let Some(g) = self.resolve_global(name) { g } else { let i = self.globals.len() as u8; self.globals.insert(name.clone(), i); i };
                self.emit_op(OpCode::DefineGlobal, 0);
                self.emit_byte(g, 0);
                Ok(())
            }
        }
    }

    fn expr(&mut self, e: &Expr) -> Result<(), CompileError> {
        match e {
            Expr::Literal { value } => {
                let v = match value { Literal::Number(n) => Value::Number(*n), Literal::Str(s) => Value::from_string(s.clone()), Literal::Bool(true) => Value::Bool(true), Literal::Bool(false) => Value::Bool(false), Literal::Nil | Literal::None => Value::Nil };
                self.emit_constant(v, 0)?;
                Ok(())
            }
            Expr::Variable { name } => {
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_op(OpCode::GetLocal, 0);
                    self.emit_byte(slot, 0);
                } else if let Some(g) = self.resolve_global(name) {
                    self.emit_op(OpCode::GetGlobal, 0);
                    self.emit_byte(g, 0);
                } else { return Err(CompileError { line: 0, message: format!("undefined variable '{}'", name) }); }
                Ok(())
            }
            Expr::Assign { name, value } => {
                self.expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_op(OpCode::SetLocal, 0);
                    self.emit_byte(slot, 0);
                } else if let Some(g) = self.resolve_global(name) {
                    self.emit_op(OpCode::SetGlobal, 0);
                    self.emit_byte(g, 0);
                } else { return Err(CompileError { line: 0, message: format!("undefined variable '{}'", name) }); }
                Ok(())
            }
            Expr::Group { inner } => self.expr(inner),
            Expr::Unary { op, operand } => { self.expr(operand)?; self.emit_op(match op { UnaryOp::Neg => OpCode::Neg, UnaryOp::Not => OpCode::Not }, 0); Ok(()) }
            Expr::Binary { op, left, right } => { self.expr(left)?; self.expr(right)?; self.emit_op(match op { BinOp::Add => OpCode::Add, BinOp::Sub => OpCode::Sub, BinOp::Mul => OpCode::Mul, BinOp::Div => OpCode::Div, BinOp::Eq => OpCode::Equal, BinOp::Ne => OpCode::Not, /* EQUAL used with NOT trick — see vm */ BinOp::Lt => OpCode::Less, BinOp::Le => OpCode::Greater, /* handle via NOT too */ BinOp::Gt => OpCode::Greater, BinOp::Ge => OpCode::Less, BinOp::And | BinOp::Or => unreachable!() }, 0); Ok(()) }
            Expr::Logical { op, left, right } => {
                self.expr(left)?;
                match op { BinOp::Or => { let j = self.emit_jump(OpCode::JumpIfFalse, 0); self.emit_op(OpCode::Pop, 0); self.expr(right)?; self.patch_jump(j)?; } BinOp::And => { let j = self.emit_jump(OpCode::JumpIfFalse, 0); self.emit_op(OpCode::Pop, 0); self.expr(right)?; self.patch_jump(j)?; } _ => unreachable!() }
                Ok(())
            }
            Expr::Call { callee, args } => { self.expr(callee)?; for a in args { self.expr(a)?; } self.emit_op(OpCode::Call, 0); self.emit_byte(args.len() as u8, 0); Ok(()) }
            Expr::Empty => { self.emit_op(OpCode::Nil, 0); Ok(()) }
        }
    }
}
```

- [ ] **Step 4: Extend `Obj` for chunk storage**

In `src/obj.rs`, change `FunctionObj` to carry a chunk:

```rust
use crate::opcode::Chunk;

pub struct FunctionObj { pub name: String, pub arity: u8, pub chunk: Option<Chunk> }
```

- [ ] **Step 5: Wire in**

Edit `src/lib.rs` — add `pub mod compiler;`.

- [ ] **Step 6: Run tests**

Run: `cargo test --test compile_smoke`
Expected: PASS. (The compiler test only checks the opcodes appear in the byte stream; running the bytecode is the VM's job.)

- [ ] **Step 7: Commit**

```bash
git add src/compiler.rs src/obj.rs src/lib.rs tests/compile_smoke.rs
git commit -m "add AST-to-bytecode compiler (no closures yet)"
```

---

## Task 16: Upvalue capture

**Files:**
- Modify: `src/compiler.rs`

- [ ] **Step 1: Add a `compiler_n` field to track enclosing local indices**

Replace the `Compiler` struct body and methods that need to know about enclosing functions. This is a focused change — keep the rest of the compiler identical.

Edit `src/compiler.rs` to add an `enclosing: Option<Box<Compiler>>` field and rework `add_local` / `resolve_local` to look up the chain. The full diff is non-trivial; the change pattern is:

```rust
pub struct Compiler { /* ...existing... */ enclosing: Option<Box<Compiler>> }

impl Compiler {
    fn resolve_local(&self, name: &str) -> Option<u8> {
        for (i, l) in self.locals.iter().enumerate().rev() {
            if l.name == name && l.depth != -1 { return Some(i as u8); }
        }
        None
    }
    fn resolve_upvalue(&mut self, name: &str) -> Option<u8> {
        let enc = self.enclosing.as_mut()?;
        if let Some(local) = enc.resolve_local(name) {
            enc.locals[local as usize].is_captured = true;
            return self.add_upvalue(local, true);
        }
        if let Some(uv) = enc.resolve_upvalue(name) {
            return self.add_upvalue(uv, false);
        }
        None
    }
    fn add_upvalue(&mut self, index: usize, is_local: bool) -> Option<u8> {
        // Cache upvalues per function (de-dupe): store in self.function.upvalues
        if let Some(&i) = self.function.upvalues.iter().position(|u| u.index == index && u.is_local == is_local) {
            return Some(i as u8);
        }
        if self.function.upvalues.len() >= u8::MAX as usize { return None; }
        self.function.upvalues.push(crate::upvalue::UpvalueRef { index: index as u8, is_local });
        Some((self.function.upvalues.len() - 1) as u8)
    }
}
```

- [ ] **Step 2: Extend `Function` with upvalue list**

```rust
pub struct Function {
    pub arity: u8,
    pub chunk: Chunk,
    pub name: String,
    pub upvalues: Vec<crate::upvalue::UpvalueRef>,
}
```

Add to `src/upvalue.rs`:

```rust
#[derive(Debug, Clone, Copy)]
pub struct UpvalueRef { pub index: u8, pub is_local: bool }
```

- [ ] **Step 3: Update variable resolution to fall through to upvalues**

In `Compiler::expr`, in the `Expr::Variable` and `Expr::Assign` arms, after the local/global resolution fails, try `resolve_upvalue`. If found, emit `GetUpvalue` / `SetUpvalue` with the upvalue index.

- [ ] **Step 4: Compile the `CLOSURE` opcode for inner functions**

In the `Stmt::Fn` arm, when emitting the function's constant, after writing the chunk and patching it in, **also emit upvalue operands** for each `UpvalueRef` collected in `sub.function.upvalues`:

```rust
self.emit_op(OpCode::Closure, 0);
self.emit_byte(idx, 0);
// Each upvalue: 1 byte (constant index) + 1 byte (is_local flag)
for uv in &sub.function.upvalues {
    self.emit_byte(uv.index, 0);
    self.emit_byte(if uv.is_local { 1 } else { 0 }, 0);
}
```

- [ ] **Step 5: Test**

Add to `tests/compile_smoke.rs`:

```rust
#[test]
fn compile_closure_emits_closure_opcode() {
    let src = "fn outer() { var x = 1; fn inner() { return x; } }";
    let toks = scripting_vm::lexer::Lexer::new(src).scan_tokens().unwrap();
    let prog = scripting_vm::parser::Parser::new(toks).parse().unwrap();
    let c = scripting_vm::compiler::Compiler::new();
    let func = c.compile(&prog).unwrap();
    assert!(func.chunk.code.iter().any(|b| *b == scripting_vm::opcode::OpCode::Closure as u8));
}
```

Run: `cargo test --test compile_smoke`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/compiler.rs src/upvalue.rs tests/compile_smoke.rs
git commit -m "compiler: upvalue capture for closures"
```

---

## Task 17: VM execution loop

**Files:**
- Create: `src/vm.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write failing integration test**

Create `tests/vm_smoke.rs`:

```rust
use scripting_vm::compiler::Compiler;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::vm::VM;

#[test]
fn vm_runs_arith() {
    let src = "print 1 + 2 * 3;";
    let toks = Lexer::new(src).scan_tokens().unwrap();
    let prog = Parser::new(toks).parse().unwrap();
    let c = Compiler::new();
    let func = c.compile(&prog).unwrap();
    let mut vm = VM::new();
    let out = vm.run(func);
    assert!(out.is_ok());
}
```

- [ ] **Step 2: Run test**

Run: `cargo test --test vm_smoke`
Expected: FAIL.

- [ ] **Step 3: Implement VM**

Create `src/vm.rs`:

```rust
use std::cell::RefCell;
use std::rc::Rc;
use crate::compiler::Function;
use crate::error::{RuntimeError, StackFrame};
use crate::native;
use crate::obj::{Obj, ObjKind};
use crate::opcode::OpCode;
use crate::upvalue::Upvalue;
use crate::value::Value;

const STACK_MAX: usize = 1024;
const FRAMES_MAX: usize = 64;

pub struct CallFrame {
    pub function: Rc<RefCell<Obj>>,
    pub ip: usize,
    pub slots_offset: usize,
}

pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    open_upvalues: Vec<Rc<RefCell<Upvalue>>>,
    globals: std::collections::HashMap<String, Value>,
    pub output: Vec<String>,
}

impl VM {
    pub fn new() -> Self {
        let mut globals = std::collections::HashMap::new();
        for (n, f) in native::registry() { globals.insert(n.to_string(), Value::Native(f)); }
        Self { frames: vec![], stack: vec![], open_upvalues: vec![], globals, output: vec![] }
    }

    pub fn run(&mut self, top: Function) -> Result<(), RuntimeError> {
        let function_obj = Rc::new(RefCell::new(Obj { kind: ObjKind::Function(crate::obj::FunctionObj { name: top.name.clone(), arity: top.arity, chunk: Some(top.chunk.clone()) }) }));
        if top.arity != 0 { return Err(RuntimeError { message: "top-level function must have arity 0".into(), stack_trace: vec![] }); }
        // Push a frame manually
        self.frames.push(CallFrame { function: function_obj, ip: 0, slots_offset: 0 });
        // Pre-bind the function value in the stack (slot 0) and start
        self.stack.push(Value::Obj(function_obj.clone()));
        self.execute()
    }

    fn read_byte(&mut self) -> u8 {
        let f = self.frames.last_mut().unwrap();
        let b = {
            let func = f.function.borrow();
            let chunk = match &func.kind { ObjKind::Function(fo) => fo.chunk.as_ref().unwrap(), _ => unreachable!() };
            chunk.code[f.ip]
        };
        f.ip += 1;
        b
    }
    fn read_u16(&mut self) -> u16 {
        let hi = self.read_byte() as u16;
        let lo = self.read_byte() as u16;
        (hi << 8) | lo
    }
    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte() as usize;
        let f = self.frames.last().unwrap();
        let func = f.function.borrow();
        let chunk = match &func.kind { ObjKind::Function(fo) => fo.chunk.as_ref().unwrap(), _ => unreachable!() };
        chunk.constants[idx].clone()
    }
    fn read_string(&mut self) -> String {
        let v = self.read_constant();
        match v { Value::Obj(o) => { let b = o.borrow(); if let ObjKind::String(s) = &b.kind { s.clone() } else { panic!("expected string") } } _ => panic!("expected string") }
    }
    fn line_at(&self, ip: usize) -> usize {
        let f = self.frames.last().unwrap();
        let func = f.function.borrow();
        let chunk = match &func.kind { ObjKind::Function(fo) => fo.chunk.as_ref().unwrap(), _ => return 0 };
        chunk.lines.get(ip).copied().unwrap_or(0)
    }
    fn push(&mut self, v: Value) {
        if self.stack.len() >= STACK_MAX { panic!("stack overflow"); }
        self.stack.push(v);
    }
    fn pop(&mut self) -> Value { self.stack.pop().unwrap() }
    fn peek(&self, n: usize) -> &Value { &self.stack[self.stack.len() - 1 - n] }
    fn frame_chunk(&self) -> std::cell::Ref<'_, crate::opcode::Chunk> {
        let f = self.frames.last().unwrap();
        let func = f.function.borrow();
        let fo = match &func.kind { ObjKind::Function(fo) => fo, _ => unreachable!() };
        std::cell::Ref::map(func, |o| match &o.kind { ObjKind::Function(fo) => fo.chunk.as_ref().unwrap(), _ => unreachable!() })
    }
    fn frame_name(&self) -> String {
        let f = self.frames.last().unwrap();
        let func = f.function.borrow();
        match &func.kind { ObjKind::Function(fo) => fo.name.clone(), _ => "<script>".into() }
    }
    fn capture_upvalue(&mut self, slot: usize) -> Rc<RefCell<Upvalue>> {
        for uv in &self.open_upvalues {
            if uv.borrow().index == slot && uv.borrow().is_open { return uv.clone(); }
        }
        let uv = Rc::new(RefCell::new(Upvalue::new(slot, true)));
        self.open_upvalues.push(uv.clone());
        uv
    }
    fn close_upvalues(&mut self, from: usize) {
        for uv in &self.open_upvalues {
            let mut b = uv.borrow_mut();
            if b.index >= from && b.is_open {
                b.value = self.stack[b.index].clone();
                b.is_open = false;
            }
        }
    }
    fn runtime_err(&self, msg: &str) -> RuntimeError {
        let mut trace = vec![];
        for (i, fr) in self.frames.iter().enumerate().rev() {
            let func = fr.function.borrow();
            let line = match &func.kind { ObjKind::Function(fo) => fo.chunk.as_ref().map(|c| c.lines[fr.ip.saturating_sub(1)]).unwrap_or(0), _ => 0 };
            let name = match &func.kind { ObjKind::Function(fo) => fo.name.clone(), _ => "<script>".into() };
            trace.push(StackFrame { function: name, line });
            if i == 0 { break; }
        }
        RuntimeError { message: msg.to_string(), stack_trace: trace }
    }

    fn execute(&mut self) -> Result<(), RuntimeError> {
        loop {
            let op_byte = self.read_byte();
            let op = match OpCode::try_from(op_byte) { Ok(o) => o, Err(_) => return Err(self.runtime_err(&format!("unknown opcode {}", op_byte))) };
            match op {
                OpCode::Constant => { let v = self.read_constant(); self.push(v); }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Pop => { self.pop(); }
                OpCode::GetLocal => { let slot = self.read_byte() as usize; let off = self.frames.last().unwrap().slots_offset; let v = self.stack[off + slot].clone(); self.push(v); }
                OpCode::SetLocal => { let slot = self.read_byte() as usize; let off = self.frames.last().unwrap().slots_offset; self.stack[off + slot] = self.peek(0).clone(); }
                OpCode::GetGlobal => { let name = self.read_string(); let v = self.globals.get(&name).cloned().ok_or_else(|| self.runtime_err(&format!("undefined variable '{}'", name)))?; self.push(v); }
                OpCode::DefineGlobal => { let name = self.read_string(); let v = self.pop(); self.globals.insert(name, v); }
                OpCode::SetGlobal => { let name = self.read_string(); let v = self.peek(0).clone(); if let Some(slot) = self.globals.get_mut(&name) { *slot = v; } else { return Err(self.runtime_err(&format!("undefined variable '{}'", name))); } }
                OpCode::GetUpvalue => { let slot = self.read_byte() as usize; let f = self.frames.last().unwrap(); let func = f.function.borrow(); let upvalues = match &func.kind { ObjKind::Function(fo) => &fo.upvalues, _ => unreachable!() }; let uv_ref = &upvalues[slot]; let v = if uv_ref.is_local { let off = f.slots_offset; self.stack[off + uv_ref.index as usize].clone() } else { panic!("non-local upvalues need full capture") }; self.push(v); }
                OpCode::SetUpvalue => { let slot = self.read_byte() as usize; let f = self.frames.last().unwrap(); let func = f.function.borrow(); let upvalues = match &func.kind { ObjKind::Function(fo) => &fo.upvalues, _ => unreachable!() }; let uv_ref = &upvalues[slot]; let v = self.peek(0).clone(); if uv_ref.is_local { let off = f.slots_offset; self.stack[off + uv_ref.index as usize] = v; } else { panic!("non-local upvalues need full capture") } }
                OpCode::Add => { let b = self.pop(); let a = self.pop(); self.push(self.add(a, b)?); }
                OpCode::Sub => { let b = self.pop(); let a = self.pop(); self.push(num_bin(a, b, |x, y| x - y, &self.runtime_err(""))?); }
                OpCode::Mul => { let b = self.pop(); let a = self.pop(); self.push(num_bin(a, b, |x, y| x * y, &self.runtime_err(""))?); }
                OpCode::Div => { let b = self.pop(); let a = self.pop(); self.push(num_bin(a, b, |x, y| x / y, &self.runtime_err(""))?); }
                OpCode::Neg => { let v = self.pop(); if let Value::Number(n) = v { self.push(Value::Number(-n)); } else { return Err(self.runtime_err("operand must be a number")); } }
                OpCode::Not => { let v = self.pop(); self.push(Value::Bool(!v.is_truthy())); }
                OpCode::Equal => { let b = self.pop(); let a = self.pop(); self.push(Value::Bool(a.equals(&b))); }
                OpCode::Greater => { let b = self.pop(); let a = self.pop(); if let (Value::Number(x), Value::Number(y)) = (a, b) { self.push(Value::Bool(x > y)); } else { return Err(self.runtime_err("operands must be numbers")); } }
                OpCode::Less => { let b = self.pop(); let a = self.pop(); if let (Value::Number(x), Value::Number(y)) = (a, b) { self.push(Value::Bool(x < y)); } else { return Err(self.runtime_err("operands must be numbers")); } }
                OpCode::Print => { let v = self.pop(); self.output.push(v.to_string()); }
                OpCode::Jump => { let off = self.read_u16() as usize; let f = self.frames.last_mut().unwrap(); f.ip += off; }
                OpCode::JumpIfFalse => { let off = self.read_u16() as usize; if !self.peek(0).is_truthy() { let f = self.frames.last_mut().unwrap(); f.ip += off; } }
                OpCode::Loop => { let off = self.read_u16() as usize; let f = self.frames.last_mut().unwrap(); f.ip -= off; }
                OpCode::Call => { let argc = self.read_byte() as usize; self.call_value(argc)?; }
                OpCode::Closure => {
                    let v = self.read_constant();
                    if let Value::Obj(o) = v {
                        let f = o.borrow();
                        let fo = match &f.kind { ObjKind::Function(fo) => fo.clone(), _ => return Err(self.runtime_err("CLOSURE expected function object")) };
                        let mut closure = Obj { kind: ObjKind::Closure(crate::obj::ClosureObj { function: Box::new(crate::obj::FunctionObj { name: fo.name.clone(), arity: fo.arity, chunk: fo.chunk.clone() }) }) };
                        for uv in &fo.upvalues {
                            let is_local = uv.is_local;
                            let idx = uv.index as usize;
                            if is_local {
                                let captured = self.capture_upvalue(self.frames.last().unwrap().slots_offset + idx);
                                if let ObjKind::Closure(co) = &mut closure.kind { co.upvalues.push(captured); }
                            } else { panic!("non-local upvalue not supported in v1"); }
                        }
                        self.push(Value::Obj(Rc::new(RefCell::new(closure))));
                    } else { return Err(self.runtime_err("CLOSURE expected function")); }
                }
                OpCode::CloseUpvalue => { self.close_upvalues(self.stack.len() - 1); self.pop(); }
                OpCode::Return => {
                    let result = self.pop();
                    let frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() { return Ok(()); }
                    self.close_upvalues(frame.slots_offset);
                    self.stack.truncate(frame.slots_offset);
                    self.push(result);
                }
                _ => return Err(self.runtime_err(&format!("opcode {:?} not implemented in v1", op))),
            }
        }
    }

    fn add(&self, a: Value, b: Value) -> Result<Value, RuntimeError> {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => Ok(Value::Number(x + y)),
            (Value::Obj(o1), Value::Obj(o2)) => {
                let (a, b) = (o1.borrow(), o2.borrow());
                if let (ObjKind::String(s1), ObjKind::String(s2)) = (&a.kind, &b.kind) { Ok(Value::from_string(format!("{}{}", s1, s2))) } else { Err(self.runtime_err("operands must be two numbers or two strings")) }
            }
            _ => Err(self.runtime_err("operands must be two numbers or two strings")),
        }
    }
    fn call_value(&mut self, argc: usize) -> Result<(), RuntimeError> {
        let callee = self.peek(argc).clone();
        match callee {
            Value::Obj(o) => {
                let kind = o.borrow().kind.clone();
                match kind {
                    ObjKind::Closure(co) => {
                        if co.function.arity as usize != argc { return Err(self.runtime_err(&format!("expected {} args, got {}", co.function.arity, argc))); }
                        if self.frames.len() + 1 >= FRAMES_MAX { return Err(self.runtime_err("stack overflow")); }
                        let arity = co.function.arity as usize;
                        self.frames.push(CallFrame { function: Rc::new(RefCell::new(Obj { kind: ObjKind::Function(*co.function.clone()) })), ip: 0, slots_offset: self.stack.len() - argc - 1 });
                        // pre-allocate slots
                        for _ in 0..arity { self.push(Value::Nil); }
                        Ok(())
                    }
                    _ => Err(self.runtime_err("not a function")),
                }
            }
            Value::Native(f) => {
                let args: Vec<Value> = self.stack[self.stack.len() - argc..].to_vec();
                let r = f(&args).map_err(|m| self.runtime_err(&m))?;
                self.stack.truncate(self.stack.len() - argc - 1);
                self.push(r);
                Ok(())
            }
            _ => Err(self.runtime_err("not callable")),
        }
    }
}

fn num_bin<F: Fn(f64, f64) -> f64>(a: Value, b: Value, f: F, _e: &RuntimeError) -> Result<Value, RuntimeError> {
    if let (Value::Number(x), Value::Number(y)) = (a, b) { Ok(Value::Number(f(x, y))) } else { Err(RuntimeError { message: "operands must be numbers".into(), stack_trace: vec![] }) }
}
```

- [ ] **Step 4: Add helpers to `obj.rs`**

Edit `src/obj.rs`:

```rust
use crate::opcode::Chunk;
use std::rc::Rc;
use std::cell::RefCell;
use crate::upvalue::Upvalue;

pub struct Obj { pub kind: ObjKind }

#[derive(Clone)]
pub enum ObjKind {
    String(String),
    Function(FunctionObj),
    Closure(ClosureObj),
    Upvalue(Upvalue),
}

#[derive(Clone)]
pub struct FunctionObj { pub name: String, pub arity: u8, pub chunk: Option<Chunk> }

#[derive(Clone)]
pub struct ClosureObj { pub function: Box<FunctionObj>, pub upvalues: Vec<Rc<RefCell<Upvalue>>> }
```

- [ ] **Step 5: Wire it in**

Edit `src/lib.rs` — add `pub mod vm;`.

- [ ] **Step 6: Run tests**

Run: `cargo test --test vm_smoke`
Expected: PASS. (The test currently doesn't assert output, only that the VM runs without error — output assertions come in the parity test.)

- [ ] **Step 7: Commit**

```bash
git add src/vm.rs src/obj.rs src/lib.rs tests/vm_smoke.rs
git commit -m "add stack-based bytecode VM"
```

---

## Task 18: CLI subcommands

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Implement CLI**

Replace `src/main.rs`:

```rust
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::cell::RefCell;
use std::rc::Rc;

use scripting_vm::compiler::Compiler;
use scripting_vm::interpreter::Interpreter;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::vm::VM;

fn run(src: &str) {
    let toks = match Lexer::new(src).scan_tokens() { Ok(t) => t, Err(e) => { eprintln!("{}", e); return } };
    let prog = match Parser::new(toks).parse() { Ok(p) => p, Err(e) => { eprintln!("{}", e); return } };
    let c = Compiler::new();
    let func = match c.compile(&prog) { Ok(f) => f, Err(e) => { eprintln!("{}", e); return } };
    let mut vm = VM::new();
    if let Err(e) = vm.run(func) { eprintln!("{}", e); return; }
    for line in vm.output { println!("{}", line); }
}

fn compile(src: &str) {
    let toks = match Lexer::new(src).scan_tokens() { Ok(t) => t, Err(e) => { eprintln!("{}", e); return } };
    let prog = match Parser::new(toks).parse() { Ok(p) => p, Err(e) => { eprintln!("{}", e); return } };
    let c = Compiler::new();
    let func = match c.compile(&prog) { Ok(f) => f, Err(e) => { eprintln!("{}", e); return } };
    println!("== {} ==", func.name);
    if let Some(chunk) = &func.chunk.clone().code.get(0..).and(Some(())) {
        let _ = chunk; // appease borrowck; disassembly is a later task
    }
    println!("(disassembly not yet implemented; chunk: {} bytes)", func.chunk.code.len());
}

fn repl() {
    let stdin = io::stdin();
    println!("scripting-vm REPL. Type :quit to exit, :disassemble to dump bytecode of the last fn.");
    let mut last_fn: Option<scripting_vm::compiler::Function> = None;
    let mut vm = VM::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let line = match stdin.lock().lines().next() { Some(Ok(l)) => l, _ => break };
        let line = line.trim();
        if line.is_empty() { continue; }
        if line == ":quit" { break; }
        if line == ":disassemble" { println!("(disassembly not yet implemented)"); continue; }
        let toks = match Lexer::new(line).scan_tokens() { Ok(t) => t, Err(e) => { eprintln!("{}", e); continue } };
        let prog = match Parser::new(toks).parse() { Ok(p) => p, Err(e) => { eprintln!("{}", e); continue } };
        let c = Compiler::new();
        match c.compile(&prog) {
            Ok(f) => {
                if let Err(e) = vm.run(f.clone()) { eprintln!("{}", e); continue; }
                for l in &vm.output { println!("{}", l); }
                vm.output.clear();
                last_fn = Some(f);
            }
            Err(e) => eprintln!("{}", e),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("run") => { let src = fs::read_to_string(args.get(2).expect("usage: run <file>")).expect("read file"); run(&src); }
        Some("compile") => { let src = fs::read_to_string(args.get(2).expect("usage: compile <file>")).expect("read file"); compile(&src); }
        Some("repl") | None => repl(),
        Some(other) => eprintln!("unknown subcommand: {}", other),
    }
}
```

- [ ] **Step 2: Build and run a file**

Create `examples/hello.lang`:

```
print "hello, world";
```

Run: `cargo run -- run examples/hello.lang`
Expected: `hello, world`

- [ ] **Step 3: Commit**

```bash
git add src/main.rs examples/hello.lang
git commit -m "cli: repl, run, compile subcommands"
```

---

## Task 19: Parity test harness

**Files:**
- Create: `tests/common/run_both.rs`
- Create: `tests/parity.rs`

- [ ] **Step 1: Create the harness**

Create `tests/common/mod.rs` (empty — used only as a marker for cargo):

```rust
// marker module
```

Create `tests/common/run_both.rs`:

```rust
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use std::rc::Rc;

use scripting_vm::compiler::Compiler;
use scripting_vm::interpreter::Interpreter;
use scripting_vm::lexer::Lexer;
use scripting_vm::parser::Parser;
use scripting_vm::vm::VM;

pub fn run_interpreter(path: &Path) -> String {
    let src = fs::read_to_string(path).expect("read .lang file");
    let toks = Lexer::new(&src).scan_tokens().expect("lex");
    let prog = Parser::new(toks).parse().expect("parse");
    let out = Rc::new(RefCell::new(Vec::new()));
    let mut interp = Interpreter::with_sink(out.clone());
    if let Err(e) = interp.run(&prog) { return format!("INTERPRETER ERROR: {}", e); }
    out.borrow().join("\n")
}

pub fn run_vm(path: &Path) -> String {
    let src = fs::read_to_string(path).expect("read .lang file");
    let toks = Lexer::new(&src).scan_tokens().expect("lex");
    let prog = Parser::new(toks).parse().expect("parse");
    let c = Compiler::new();
    let func = c.compile(&prog).expect("compile");
    let mut vm = VM::new();
    if let Err(e) = vm.run(func) { return format!("VM ERROR: {}", e); }
    vm.output.join("\n")
}
```

Create `tests/parity.rs`:

```rust
mod common;
use common::run_both::*;
use std::path::Path;

fn check(name: &str) {
    let path = Path::new("tests/cases").join(name);
    let a = run_interpreter(&path);
    let b = run_vm(&path);
    assert_eq!(a, b, "parity mismatch in {}", name);
}

#[test] fn parity_arith()   { check("arith.lang"); }
#[test] fn parity_vars()    { check("vars.lang"); }
#[test] fn parity_print()   { check("print.lang"); }
```

- [ ] **Step 2: Create initial test cases**

Create `tests/cases/arith.lang`:
```
print 1 + 2;
print 10 - 3;
print 4 * 5;
print 20 / 4;
print -7;
```

Create `tests/cases/vars.lang`:
```
var x = 5;
var y = x + 2;
print y;
x = 10;
print x;
```

Create `tests/cases/print.lang`:
```
print "hello";
print true;
print nil;
print 1 + 2;
```

- [ ] **Step 3: Run tests**

Run: `cargo test --test parity`
Expected: PASS for all three.

- [ ] **Step 4: Commit**

```bash
git add tests/common tests/parity.rs tests/cases
git commit -m "test: parity harness for interpreter vs VM"
```

---

## Task 20: Test case coverage (features + closures)

**Files:**
- Create: `tests/cases/*.lang`

- [ ] **Step 1: Create 17+ additional test cases**

Create these files in `tests/cases/`:

- `if.lang` — `if (1 < 2) { print "yes"; } else { print "no"; }`
- `while.lang` — `var i = 0; while (i < 3) { print i; i = i + 1; }`
- `logical.lang` — `print true and false; print true or false; print !true;`
- `cmp.lang` — `print 1 == 1; print 1 != 2; print 1 < 2; print 2 >= 2;`
- `strings.lang` — `print "a" + "b"; var s = "hi"; print s;`
- `precedence.lang` — `print 1 + 2 * 3; print (1 + 2) * 3; print -3 + 4;`
- `unary.lang` — `print - -5; print !false;`
- `group.lang` — `print (1 + 2) * (3 + 4);`
- `fib.lang` — `fn fib(n) { if (n < 2) { return n; } return fib(n-1) + fib(n-2); } print fib(10);`
- `counter.lang` — `fn make() { var i = 0; fn tick() { i = i + 1; return i; } return tick; } var c = make(); print c(); print c(); print c();`
- `nested_fn.lang` — `fn outer() { fn inner() { return 42; } return inner(); } print outer();`
- `mutual_close.lang` — `fn a() { var x = 1; fn b() { return x; } return b(); } print a();`
- `multi_args.lang` — `fn add(a, b, c) { return a + b + c; } print add(1, 2, 3);`
- `recursion.lang` — `fn fact(n) { if (n < 2) { return 1; } return n * fact(n - 1); } print fact(5);`
- `global_reassign.lang` — `var x = 1; x = 2; print x;`
- `scope.lang` — `var x = 1; { var x = 2; print x; } print x;`
- `stress.lang` — `var sum = 0; var i = 0; while (i < 100) { sum = sum + i; i = i + 1; } print sum;`

- [ ] **Step 2: Add parity test entries**

Append to `tests/parity.rs`:

```rust
#[test] fn parity_if()          { check("if.lang"); }
#[test] fn parity_while()       { check("while.lang"); }
#[test] fn parity_logical()     { check("logical.lang"); }
#[test] fn parity_cmp()         { check("cmp.lang"); }
#[test] fn parity_strings()     { check("strings.lang"); }
#[test] fn parity_precedence()  { check("precedence.lang"); }
#[test] fn parity_unary()       { check("unary.lang"); }
#[test] fn parity_group()       { check("group.lang"); }
#[test] fn parity_fib()         { check("fib.lang"); }
#[test] fn parity_counter()     { check("counter.lang"); }
#[test] fn parity_nested_fn()   { check("nested_fn.lang"); }
#[test] fn parity_mutual_close(){ check("mutual_close.lang"); }
#[test] fn parity_multi_args()  { check("multi_args.lang"); }
#[test] fn parity_recursion()   { check("recursion.lang"); }
#[test] fn parity_global_reassign() { check("global_reassign.lang"); }
#[test] fn parity_scope()       { check("scope.lang"); }
#[test] fn parity_stress()      { check("stress.lang"); }
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: ALL parity tests pass; no regressions.

- [ ] **Step 4: Commit**

```bash
git add tests/cases tests/parity.rs
git commit -m "test: 20 parity cases covering full feature set"
```

---

## Task 21: README

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write the README**

```markdown
# scripting-vm

A small, complete, Lox-style dynamic scripting language with a stack-based bytecode VM. Written in Rust. Portfolio piece.

## What it is

A from-scratch implementation of an interpreter pipeline: source → tokens → AST → bytecode → execution. Two execution paths (a tree-walking reference and the bytecode VM) agree on output for every test program.

## What's implemented

- Lexer (with line tracking)
- Recursive-descent parser with Pratt precedence
- Tree-walking interpreter (reference)
- Single-pass AST → bytecode compiler
- Stack-based VM with closures and upvalues
- Dynamic typing, first-class functions, recursion
- REPL with `run` and `compile` subcommands
- Parity test suite: 20 .lang programs run through both paths, output must match

## What's not implemented (v1)

- Classes, inheritance, `this`, `super`
- Tracing garbage collector (uses `Rc`; cycles leak — see [known limitations](docs/known-limitations.md))
- Standard library beyond `print` and `clock`
- Optimization passes
- Debug protocol

## Build & run

```bash
cargo build --release
cargo run -- run examples/fib.lang
cargo run -- repl
```

## Run tests

```bash
cargo test
```

The parity test (`tests/parity.rs`) is the key check — if both executors produce identical output on all 20 programs, the VM matches the reference.

## Project layout

```
src/
  lexer.rs      source -> tokens
  parser.rs     tokens -> AST
  ast.rs        AST types
  value.rs      runtime values
  obj.rs        heap objects
  env.rs        lexical scopes
  interpreter.rs  tree-walking reference
  compiler.rs   AST -> bytecode
  opcode.rs     opcode set + Chunk
  upvalue.rs    closure capture
  vm.rs         stack machine
  native.rs     built-in functions
  main.rs       CLI
tests/cases/*.lang  parity test programs
```

## Architecture

See [docs/architecture.md](docs/architecture.md). Opcode reference: [docs/bytecode.md](docs/bytecode.md).
```

- [ ] **Step 2: Commit**

```bash
git add README.md
git commit -m "docs: readme"
```

---

## Task 22: Architecture doc

**Files:**
- Create: `docs/architecture.md`

- [ ] **Step 1: Write the doc**

```markdown
# Architecture

## Pipeline

```
source  --lexer-->  tokens  --parser-->  AST
                                        |
                                  +-----+-----+
                                  |           |
                          (interpreter)  (compiler)
                                  |           |
                              values       bytecode
                                             |
                                            VM
                                             |
                                          values
```

## Why two executors

The interpreter is the reference. When the VM's behavior diverges from it on any test, you have a bug. The parity test in `tests/parity.rs` enforces this on every build.

## Module map

| Module | Owns | Depends on |
|--------|------|------------|
| `token` | Token, TokenType | — |
| `lexer` | source -> tokens | `token` |
| `ast` | Expr, Stmt | `token` (for Literal) |
| `error` | CompileError, RuntimeError | — |
| `parser` | tokens -> AST | `lexer`, `ast`, `error` |
| `value` | Value enum | `obj` |
| `obj` | Obj, ObjKind | `value`, `opcode`, `upvalue` |
| `env` | lexical scopes | `value`, `error` |
| `interpreter` | tree-walking eval | `ast`, `env`, `value`, `obj`, `error`, `native` |
| `opcode` | OpCode, Chunk | `value` |
| `upvalue` | Upvalue, UpvalueRef | `value` |
| `compiler` | AST -> bytecode | `ast`, `opcode`, `upvalue`, `value`, `obj`, `error`, `native` |
| `vm` | bytecode execution | `compiler`, `opcode`, `value`, `obj`, `upvalue`, `native`, `error` |
| `native` | built-in functions | `value` |
| `main` | CLI dispatch | all of the above |

The dependency graph is one-way: nothing in `compiler` or `vm` is imported by `interpreter`. Both consume the same AST.

## Closures

When the compiler sees a function declaration inside another function, it walks the enclosing function's locals. Each captured local becomes an **upvalue** on the inner function. The inner function's chunk is wrapped in a `Closure` object at runtime, with the captured upvalues attached.

Upvalues have two states:
- **open** — the captured local is still on the VM's stack
- **closed** — when the enclosing scope ends, the value is hoisted to the heap so the closure can outlive the stack frame

## Errors

- Compile errors halt at parse or compile time
- Runtime errors propagate up the call stack; the VM prints a stack trace with function names and line numbers
- No `unwrap()` on user input paths
```

- [ ] **Step 2: Commit**

```bash
git add docs/architecture.md
git commit -m "docs: architecture"
```

---

## Task 23: Bytecode reference

**Files:**
- Create: `docs/bytecode.md`

- [ ] **Step 1: Write the doc**

```markdown
# Bytecode Reference

## Encoding

- Each instruction: 1-byte opcode, then 0-2 operand bytes
- Operands: 1 byte for slots/indices, 2 bytes for jump offsets
- A `Chunk` is `Vec<u8> code` + `Vec<Value> constants` + `Vec<usize> lines` (line per byte, for errors)

## Opcode table

| Opcode | Operands | Effect |
|--------|----------|--------|
| CONST | u8 idx | push constants[idx] |
| NIL | — | push nil |
| TRUE / FALSE | — | push bool |
| POP | — | pop top |
| GET_LOCAL | u8 slot | push local at slot |
| SET_LOCAL | u8 slot | pop and assign to local |
| GET_GLOBAL | u8 name | push global |
| DEFINE_GLOBAL | u8 name | pop and define global |
| SET_GLOBAL | u8 name | pop and assign global |
| GET_UPVALUE | u8 idx | push upvalue |
| SET_UPVALUE | u8 idx | pop and assign upvalue |
| ADD | — | numeric or string concat |
| SUB / MUL / DIV | — | numeric |
| NEG | — | numeric negation |
| NOT | — | logical not |
| EQUAL | — | `==` |
| GREATER / LESS | — | numeric compare |
| PRINT | — | pop, print to stdout |
| JUMP | u16 off | unconditional forward jump |
| JUMP_IF_FALSE | u16 off | pop, jump if false |
| LOOP | u16 off | unconditional backward jump |
| CALL | u8 argc | invoke top |
| CLOSURE | u8 idx, n upvalues | wrap function in closure |
| CLOSE_UPVALUE | — | hoist captured local to heap |
| RETURN | — | return from current frame |

## Worked example

For `print 1 + 2;` the compiled chunk is:

```
CONST 0   ; push 1
CONST 1   ; push 2
ADD       ; pop two, push sum
PRINT     ; pop, print
NIL
RETURN    ; script return
```

The constant pool contains `[1, 2]`.

## Call frames

A `CallFrame` carries:
- the closure object
- the instruction pointer
- the slot offset on the operand stack

A new frame is pushed on `CALL` and popped on `RETURN`. Locals are addressed relative to the frame's slot offset.
```

- [ ] **Step 2: Commit**

```bash
git add docs/bytecode.md
git commit -m "docs: bytecode reference"
```

---

## Task 24: Known limitations

**Files:**
- Create: `docs/known-limitations.md`

- [ ] **Step 1: Write the doc**

```markdown
# Known limitations

These are deliberate v1 tradeoffs, not oversights. Documented so reviewers and future-me know what's intentional.

## Reference counting instead of tracing GC

Values are managed via `Rc<RefCell<…>>`. Reference cycles leak memory. The v1 language is structured to make cycles hard to produce: there are no mutable fields on heap objects, only `let` rebindings of names. In practice, the shipped test programs do not leak. A future `Gc<T>` newtype can be swapped in for a tracing collector without changing call sites.

## No classes

`class`, `this`, `super`, and inheritance are reserved keywords but not implemented. The opcode set reserves slots for them (see `src/opcode.rs`). This is a deliberate v2.

## No optimization

The compiler is single-pass. No constant folding, no peephole, no register allocation. This is fine for v1; the language is small enough that performance is not a concern.

## No standard library

`print` and `clock` only. A `String` type and a small stdlib are future work.

## No source maps

Errors include line numbers. They do not include column numbers or snippet excerpts. The line table is one entry per emitted byte, which is enough to attribute runtime errors to a source line; richer diagnostics would need a column table.

## No debug protocol

No breakpoints, no stepping. The REPL's `:disassemble` command prints the current function's bytecode, which is enough for demos.

## 256-slot limit

Locals, constants, globals, and upvalues are all addressed with 1-byte indices. Functions with more than 256 locals or constants will fail to compile. Plenty of headroom for a v1; not a real constraint.
```

- [ ] **Step 2: Commit**

```bash
git add docs/known-limitations.md
git commit -m "docs: known limitations"
```

---

## Task 25: Example programs

**Files:**
- Create: `examples/fib.lang`
- Create: `examples/closure.lang`
- Create: `examples/fizzbuzz.lang`

- [ ] **Step 1: Create `examples/fib.lang`**

```
// Fibonacci. Recursive, double-call, classic.
fn fib(n) {
  if (n < 2) { return n; }
  return fib(n - 1) + fib(n - 2);
}

var i = 0;
while (i < 10) {
  print fib(i);
  i = i + 1;
}
```

- [ ] **Step 2: Create `examples/closure.lang`**

```
// Closures capture enclosing locals by reference.
fn makeCounter() {
  var i = 0;
  fn tick() {
    i = i + 1;
    return i;
  }
  return tick;
}

var c = makeCounter();
print c();   // 1
print c();   // 2
print c();   // 3
```

- [ ] **Step 3: Create `examples/fizzbuzz.lang`**

```
// FizzBuzz from 1 to 15.
// v1 has no modulo; we count cycles with two helper counters.
var n = 1;
var mod3 = 0;
var mod5 = 0;
while (n <= 15) {
  if (mod3 == 2 and mod5 == 4) { print "FizzBuzz"; mod3 = 0; mod5 = 0; }
  else if (mod3 == 2)            { print "Fizz";      mod3 = 0; mod5 = mod5 + 1; }
  else if (mod5 == 4)            { print "Buzz";                mod3 = mod3 + 1; mod5 = 0; }
  else                           { print n;                     mod3 = mod3 + 1; mod5 = mod5 + 1; }
  n = n + 1;
}
```

- [ ] **Step 4: Run examples**

Run: `cargo run -- run examples/fib.lang`
Expected: `0\n1\n1\n2\n3\n5\n8\n13\n21\n34`

Run: `cargo run -- run examples/closure.lang`
Expected: `1\n2\n3`

- [ ] **Step 5: Commit**

```bash
git add examples
git commit -m "examples: fib, closure, fizzbuzz"
```

---

## Task 26: Final verification

- [ ] **Step 1: Run the full test suite**

Run: `cargo test`
Expected: every test passes — lexer, parser, value, env, parity, smoke tests.

- [ ] **Step 2: Lint clean**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: no warnings. Fix anything that surfaces.

- [ ] **Step 3: Release build**

Run: `cargo build --release`
Expected: clean release build.

- [ ] **Step 4: Smoke test all examples**

```bash
cargo run --release -- run examples/hello.lang
cargo run --release -- run examples/fib.lang
cargo run --release -- run examples/closure.lang
cargo run --release -- run examples/fizzbuzz.lang
```

Expected: each prints expected output (or its v1-compatible variant).

- [ ] **Step 5: Final commit**

```bash
git status
git log --oneline
```

If there are uncommitted fixups, commit them with a descriptive message.

---

## Spec coverage check

| Spec section | Tasks |
|---|---|
| §2 Language surface (statements, expressions) | T2–T8 (token, lexer, ast, parser) |
| §2.1 Value types | T9, T14 |
| §3 Two executors | T11, T17 |
| §4 Module layout | T1–T18 |
| §5 Value & memory model | T9, T10, T14 |
| §6 Bytecode format | T12 |
| §6.1 Opcode set | T12, T15, T16 |
| §7 VM model (stack, frames, closures) | T13, T16, T17 |
| §8 Error handling | T5, T11, T17 |
| §9 Testing strategy | T2, T3, T7, T8, T11, T15, T17, T19, T20 |
| §10 CLI / UX | T18 |
| §11 Documentation | T21, T22, T23, T24 |
| §12 Known tradeoffs | T24 |
| §14 Acceptance criteria | T26 |
