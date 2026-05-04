use std::io::{self, BufRead, Write};

use crate::cmd::Command;
use crate::executor::{ExecuteResult, execute};
use crate::storage::{MutationOutcome, Storage, StoredValue};

/// Run the REPL loop until EOF.
///
/// Reads lines from `input`, parses to Command, executes against `storage`,
/// formats the result, and writes to `output`. Empty lines are skipped.
/// Parse and execute errors are displayed inline (Redis-style "ERR ...")
/// and the loop continues. Only IO errors abort.
pub fn run<R: BufRead, W: Write, S: Storage>(
    mut input: R,
    mut output: W,
    storage: &mut S,
) -> io::Result<()> {
    let mut line = String::new();
    loop {
        // Prompt — must flush so the user sees "> " before they type.
        write!(output, "> ")?;
        output.flush()?;

        // Read one line. read_line returns the byte count;
        // 0 means EOF (the input is exhausted), which is our exit signal.
        line.clear();
        let bytes_read = input.read_line(&mut line)?;
        if bytes_read == 0 {
            return Ok(());
        }

        // Skip blank lines silently.
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse → Execute → Format. Each error is displayed, never propagated.
        match trimmed.parse::<Command>() {
            Err(e) => writeln!(output, "ERR {}", e)?,
            Ok(command) => match execute(storage, command) {
                Err(e) => writeln!(output, "ERR {}", e)?,
                Ok(result) => writeln!(output, "{}", format_result(&result))?,
            },
        }
    }
}

fn format_result(result: &ExecuteResult) -> String {
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
    use crate::storage::InMemoryStorage;

    #[test]
    fn run_set_then_get_outputs_stored_value() {
        // Keystone test: full Read-Eval-Print cycle. Set then Get,
        // confirm both outputs appear in order. Captures stdout via Vec<u8>.
        let input: &[u8] = b"set foo bar\nget foo\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("OK"), "output was: {}", output_str);
        assert!(output_str.contains("\"bar\""), "output was: {}", output_str,);
    }

    #[test]
    fn run_on_empty_input_terminates_cleanly() {
        // EOF semantics: empty input means immediate exit, no loop iterations.
        // Output should contain only the initial prompt, then EOF detection
        // returns Ok(()).
        let input: &[u8] = b"";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        let result = run(input, &mut output, &mut storage);

        assert!(result.is_ok());
    }

    #[test]
    fn run_invalid_command_emits_err_and_continues() {
        // Locks the "errors don't crash the loop" invariant: an unknown
        // verb produces ERR output, then the next valid line still works.
        let input: &[u8] = b"foozle bar\nset foo bar\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ERR"), "output was: {}", output_str);
        assert!(output_str.contains("OK"), "output was: {}", output_str);
    }

    #[test]
    fn run_blank_line_emits_no_response() {
        // Locks: a blank line is silently skipped — no ERR, no OK, no extra
        // output between the two prompts.
        let input: &[u8] = b"\nset foo bar\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(!output_str.contains("ERR"), "output was: {}", output_str);
        assert_eq!(
            output_str.matches("OK").count(),
            1,
            "expected exactly one OK, output was: {}",
            output_str,
        );
    }

    #[test]
    fn run_multiple_commands_processed_in_order() {
        // Locks the full Redis-like format across a multi-command session,
        // including order, prompts between lines, and trailing prompt at EOF.
        let input: &[u8] = b"set foo bar\nget foo\nexists foo\ndel foo\nget foo\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "> OK\n> \"bar\"\n> 1\n> OK\n> (nil)\n> ");
    }

    #[test]
    fn run_continues_after_execute_error() {
        // Locks: execute errors (not just parse errors) keep the loop alive.
        // Set str → Incr fails (NotAnInteger) → next valid Set still works.
        let input: &[u8] = b"set foo hello\nincr foo\nset bar 5\nincr bar 3\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ERR"), "output was: {}", output_str);
        // 8 = result of incr bar 3 after set bar 5 (which from_text typed as Int(5))
        assert!(output_str.contains("8"), "output was: {}", output_str);
    }

    #[test]
    fn run_get_str_outputs_quoted() {
        // Locks: StoredValue::Str renders with surrounding quotes.
        let input: &[u8] = b"set foo bar\nget foo\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "> OK\n> \"bar\"\n> ");
    }

    #[test]
    fn run_get_int_outputs_unquoted() {
        // Locks: StoredValue::Int renders as a bare number, no quotes.
        // "5" goes through from_text as Int(5), so Get returns Int(5).
        let input: &[u8] = b"set foo 5\nget foo\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "> OK\n> 5\n> ");
    }

    #[test]
    fn run_get_missing_outputs_nil() {
        // Locks: Read(None) renders as exact "(nil)".
        let input: &[u8] = b"get nope\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "> (nil)\n> ");
    }

    #[test]
    fn run_exists_outputs_one_for_present_zero_for_missing() {
        // Locks: Existence(true) → "1", Existence(false) → "0" (Redis-like).
        let input: &[u8] = b"set foo bar\nexists foo\nexists nope\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "> OK\n> 1\n> 0\n> ");
    }

    #[test]
    fn run_incr_on_missing_outputs_counter_value() {
        // Locks: MutationOutcome::Counter renders as a bare number.
        // counter doesn't exist → starts at 0 → +5 → 5.
        let input: &[u8] = b"incr counter 5\n";
        let mut output: Vec<u8> = Vec::new();
        let mut storage = InMemoryStorage::new();

        run(input, &mut output, &mut storage).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "> 5\n> ");
    }
}
