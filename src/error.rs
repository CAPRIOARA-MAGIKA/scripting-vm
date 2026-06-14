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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_contains_line() {
        let e = CompileError {
            line: 7,
            message: "bad".into(),
        };
        assert!(e.to_string().contains("line 7"));
    }
}
