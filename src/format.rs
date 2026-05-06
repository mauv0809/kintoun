//! Output formatting for ExecuteResult.
//!
//! Produces a Redis-like rendering used by both the REPL (`repl::run`) and
//! the TCP server (`server::connection`). Format is locked by tests in this
//! module and by REPL integration tests in `repl.rs`.
//!
//! See ADR 0010 for the format rationale.

use crate::executor::ExecuteResult;
use crate::storage::{MutationOutcome, StoredValue};

/// Render any error as the wire `ERR <msg>` form (per ADR 0010).
pub fn format_error<E: std::error::Error>(err: &E) -> String {
    format!("ERR {}", err)
}

/// Render an ExecuteResult as a Redis-like string.
///
/// Returns the value WITHOUT a trailing newline. Callers decide:
/// REPL writes via `writeln!`; TCP server packs into a frame body.
///
/// Format (per ADR 0010):
/// - `Mutation::Stored` / `Deleted` → "OK"
/// - `Mutation::Counter`            → bare number
/// - `Read(None)`                   → "(nil)"
/// - `Read(Str(s))`                 → quoted: "\"<s>\""
/// - `Read(Int(n))`                 → bare number
/// - `Existence(true)`              → "1"
/// - `Existence(false)`             → "0"
pub fn format_result(result: &ExecuteResult) -> String {
    match result {
        ExecuteResult::Mutation(MutationOutcome::Stored) => "OK".to_string(),
        ExecuteResult::Mutation(MutationOutcome::Deleted) => "OK".to_string(),
        ExecuteResult::Mutation(MutationOutcome::Counter { new_value }) => new_value.to_string(),
        ExecuteResult::Read(None) => "(nil)".to_string(),
        ExecuteResult::Read(Some(StoredValue::Str(s))) => format!("\"{}\"", s),
        ExecuteResult::Read(Some(StoredValue::Int(n))) => n.to_string(),
        ExecuteResult::Existence(true) => "1".to_string(),
        ExecuteResult::Existence(false) => "0".to_string(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutation_stored_formats_as_ok() {
        let result = ExecuteResult::Mutation(MutationOutcome::Stored);
        assert_eq!(format_result(&result), "OK");
    }
    #[test]
    fn mutation_deleted_formats_as_ok() {
        let result = ExecuteResult::Mutation(MutationOutcome::Deleted);
        assert_eq!(format_result(&result), "OK");
    }
    #[test]
    fn mutation_counter_formats_as_bare_number() {
        let result = ExecuteResult::Mutation(MutationOutcome::Counter { new_value: 42 });
        assert_eq!(format_result(&result), "42");
    }

    #[test]
    fn mutation_counter_zero_formats_as_zero() {
        // Boundary: counter at 0 (just-incremented-from-missing case).
        let result = ExecuteResult::Mutation(MutationOutcome::Counter { new_value: 0 });
        assert_eq!(format_result(&result), "0");
    }

    #[test]
    fn read_none_formats_as_nil_marker() {
        let result = ExecuteResult::Read(None);
        assert_eq!(format_result(&result), "(nil)");
    }

    #[test]
    fn read_str_formats_quoted() {
        let result = ExecuteResult::Read(Some(StoredValue::Str("hi".to_string())));
        assert_eq!(format_result(&result), "\"hi\"");
    }

    #[test]
    fn read_str_empty_formats_as_empty_quotes() {
        // Distinguishes from "(nil)" — locks the meaningful gap between
        // "key missing" and "key set to empty string". (Per ADR 0010.)
        let result = ExecuteResult::Read(Some(StoredValue::Str(String::new())));
        assert_eq!(format_result(&result), "\"\"");
    }

    #[test]
    fn read_int_formats_as_bare_number() {
        let result = ExecuteResult::Read(Some(StoredValue::Int(7)));
        assert_eq!(format_result(&result), "7");
    }

    #[test]
    fn existence_true_formats_as_one() {
        assert_eq!(format_result(&ExecuteResult::Existence(true)), "1");
    }

    #[test]
    fn existence_false_formats_as_zero() {
        assert_eq!(format_result(&ExecuteResult::Existence(false)), "0");
    }
}
