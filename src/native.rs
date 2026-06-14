use crate::value::{NativeFn, Value};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn clock(_args: &[Value]) -> Result<Value, String> {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs_f64();
    Ok(Value::Number(secs))
}

pub fn print_native(_args: &[Value]) -> Result<Value, String> {
    // print is a side-effect, not a value-returning native. The VM's Print
    // opcode uses println directly. This function is here for the registry
    // but is intentionally not exposed to user code via a global binding.
    Err("print is not callable; use the print statement".into())
}

pub fn registry() -> Vec<(&'static str, NativeFn)> {
    vec![("clock", clock as NativeFn)]
}
