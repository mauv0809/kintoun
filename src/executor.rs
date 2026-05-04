use crate::cmd::Command;
use crate::storage::{Mutation, MutationOutcome, Storage, StorageError, StoredValue};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ExecuteError {
    #[error(transparent)]
    Storage(#[from] StorageError),
}

/// Outcome of running a Command against Storage.
///
/// Three top-level variants mirror the structural read/mutation/existence
/// split that ADR 0005 locks at the storage layer. Two-level matching
/// (outer here, inner on MutationOutcome) keeps mutation outcomes living
/// in one place — storage — without ExecuteResult mirroring them.
#[derive(Debug, PartialEq)]
pub enum ExecuteResult {
    Mutation(MutationOutcome), // Set, Del, Incr, Decr
    Read(Option<StoredValue>), // Get
    Existence(bool),           // Exists
}

fn apply_and_wrap<S: Storage>(
    storage: &mut S,
    mutation: Mutation,
) -> Result<ExecuteResult, ExecuteError> {
    Ok(ExecuteResult::Mutation(storage.apply(mutation)?))
}

/// Execute a parsed Command against Storage.
///
/// `Set` values are coerced from text via `StoredValue::from_text` before
/// applying — this is the single point where text becomes typed. All other
/// commands pass through to the matching Storage method.
pub fn execute<S: Storage>(
    storage: &mut S,
    command: Command,
) -> Result<ExecuteResult, ExecuteError> {
    match command {
        Command::Set { key, value } => {
            let mutation = Mutation::Set {
                key,
                value: StoredValue::from_text(&value),
            };
            apply_and_wrap(storage, mutation)
        }
        Command::Get { key } => {
            let value = storage.read(&key);
            Ok(ExecuteResult::Read(value))
        }
        Command::Del { key } => {
            let mutation = Mutation::Del { key };
            apply_and_wrap(storage, mutation)
        }
        Command::Exists { key } => {
            let exists = storage.exists(&key);
            Ok(ExecuteResult::Existence(exists))
        }
        Command::Incr { key, by } => {
            let mutation = Mutation::Incr { key, by };
            apply_and_wrap(storage, mutation)
        }
        Command::Decr { key, by } => {
            let mutation = Mutation::Decr { key, by };
            apply_and_wrap(storage, mutation)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::InMemoryStorage;
    use pretty_assertions::assert_eq;

    #[test]
    fn execute_set_returns_mutation_stored() {
        let mut storage = InMemoryStorage::new();

        let result = execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("bar"),
            },
        );

        assert_eq!(result, Ok(ExecuteResult::Mutation(MutationOutcome::Stored)),);
    }

    #[test]
    fn execute_get_on_missing_key_returns_read_none() {
        let mut storage = InMemoryStorage::new();

        let result = execute(
            &mut storage,
            Command::Get {
                key: String::from("foo"),
            },
        );

        assert_eq!(result, Ok(ExecuteResult::Read(None)));
    }

    #[test]
    fn execute_incr_after_set_with_numeric_text_returns_counter() {
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("counter"),
                value: String::from("5"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Incr {
                key: String::from("counter"),
                by: 3,
            },
        );

        assert_eq!(
            result,
            Ok(ExecuteResult::Mutation(MutationOutcome::Counter {
                new_value: 8,
            })),
        );
    }

    #[test]
    fn execute_get_after_set_returns_stored_value() {
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("bar"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Get {
                key: String::from("foo"),
            },
        );

        assert_eq!(
            result,
            Ok(ExecuteResult::Read(Some(StoredValue::Str(String::from(
                "bar"
            ))))),
        );
    }

    #[test]
    fn execute_exists_after_set_returns_true() {
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("bar"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Exists {
                key: String::from("foo"),
            },
        );

        assert_eq!(result, Ok(ExecuteResult::Existence(true)));
    }

    #[test]
    fn execute_set_with_numeric_text_stores_int() {
        // Locks executor-level inference: numeric text → Int variant.
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("42"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Get {
                key: String::from("foo"),
            },
        );

        assert_eq!(result, Ok(ExecuteResult::Read(Some(StoredValue::Int(42)))),);
    }

    #[test]
    fn execute_set_with_non_numeric_stores_str() {
        // Locks the inference fall-through: non-numeric text → Str variant.
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("hello"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Get {
                key: String::from("foo"),
            },
        );

        assert_eq!(
            result,
            Ok(ExecuteResult::Read(Some(StoredValue::Str(String::from(
                "hello"
            ))))),
        );
    }

    #[test]
    fn execute_del_after_set_removes_value() {
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("bar"),
            },
        )
        .unwrap();
        execute(
            &mut storage,
            Command::Del {
                key: String::from("foo"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Get {
                key: String::from("foo"),
            },
        );

        assert_eq!(result, Ok(ExecuteResult::Read(None)));
    }

    #[test]
    fn execute_del_on_missing_is_idempotent() {
        let mut storage = InMemoryStorage::new();

        let result = execute(
            &mut storage,
            Command::Del {
                key: String::from("never_set"),
            },
        );

        assert_eq!(
            result,
            Ok(ExecuteResult::Mutation(MutationOutcome::Deleted)),
        );
    }

    #[test]
    fn execute_incr_on_missing_key_starts_from_zero() {
        // Executor preserves storage's "missing → 0" counter semantic.
        let mut storage = InMemoryStorage::new();

        let result = execute(
            &mut storage,
            Command::Incr {
                key: String::from("counter"),
                by: 5,
            },
        );

        assert_eq!(
            result,
            Ok(ExecuteResult::Mutation(MutationOutcome::Counter {
                new_value: 5,
            })),
        );
    }

    #[test]
    fn execute_decr_after_set_subtracts() {
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("counter"),
                value: String::from("10"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Decr {
                key: String::from("counter"),
                by: 3,
            },
        );

        assert_eq!(
            result,
            Ok(ExecuteResult::Mutation(MutationOutcome::Counter {
                new_value: 7,
            })),
        );
    }

    #[test]
    fn execute_incr_on_str_value_returns_not_an_integer_error() {
        // Error propagation: StorageError::NotAnInteger surfaces wrapped
        // as ExecuteError::Storage via the #[from] impl.
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("foo"),
                value: String::from("hello"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Incr {
                key: String::from("foo"),
                by: 1,
            },
        );

        assert_eq!(
            result,
            Err(ExecuteError::Storage(StorageError::NotAnInteger)),
        );
    }

    #[test]
    fn execute_decr_underflow_returns_underflow_error() {
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("counter"),
                value: String::from("0"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Decr {
                key: String::from("counter"),
                by: 1,
            },
        );

        assert_eq!(result, Err(ExecuteError::Storage(StorageError::Underflow)),);
    }

    #[test]
    fn execute_incr_overflow_returns_overflow_error() {
        // Pairs with the underflow test. Sets a key to u64::MAX via the
        // executor (proving from_text accepts the boundary), then the Incr
        // overflows in storage and the error propagates through executor.
        let mut storage = InMemoryStorage::new();

        execute(
            &mut storage,
            Command::Set {
                key: String::from("counter"),
                value: String::from("18446744073709551615"),
            },
        )
        .unwrap();

        let result = execute(
            &mut storage,
            Command::Incr {
                key: String::from("counter"),
                by: 1,
            },
        );

        assert_eq!(result, Err(ExecuteError::Storage(StorageError::Overflow)),);
    }
}
