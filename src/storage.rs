use std::collections::HashMap;

// Mutation - the canonical event type
// Every state-changing operation passes through here
#[derive(Debug, PartialEq, Clone)]
pub enum Mutation {
    Set { key: String, value: StoredValue },
    Del { key: String },
    Incr { key: String, by: u64 },
    Decr { key: String, by: u64 },
}

//MutationOutcome - what apply() returns on success
// Initial variants; will grow as TDD locks per-mutation semantics
#[derive(Debug, PartialEq, Clone)]
pub enum MutationOutcome {
    Stored,
    Deleted,
    Counter { new_value: u64 },
}

//Will add more when writing tests
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum StorageError {
    #[error("not an integer")]
    NotAnInteger,
    #[error("counter overflow")]
    Overflow,
    #[error("counter underflow")]
    Underflow,
}

//Storage - contrat every backend must satisfy
pub trait Storage {
    fn apply(&mut self, mutation: Mutation) -> Result<MutationOutcome, StorageError>;
    fn read(&self, key: &str) -> Option<StoredValue>;
    fn exists(&self, key: &str) -> bool;
}
#[derive(Debug, PartialEq, Clone)]
pub enum StoredValue {
    Str(String),
    Int(u64),
}
impl From<&str> for StoredValue {
    fn from(s: &str) -> Self {
        StoredValue::Str(s.to_string())
    }
}
impl From<u64> for StoredValue {
    fn from(n: u64) -> Self {
        StoredValue::Int(n)
    }
}
// InMemoryStorage
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    data: HashMap<String, StoredValue>,
}
impl InMemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
    fn apply_counter_op<F>(&mut self, key: String, op: F) -> Result<MutationOutcome, StorageError>
    where
        F: FnOnce(u64) -> Result<u64, StorageError>,
    {
        let current: u64 = match self.data.get(&key) {
            Some(StoredValue::Int(n)) => *n,
            Some(StoredValue::Str(_)) => return Err(StorageError::NotAnInteger),
            None => 0,
        };
        let new_value = op(current)?;
        self.data.insert(key, StoredValue::Int(new_value));
        Ok(MutationOutcome::Counter { new_value })
    }
}

//Helper functions:

impl Storage for InMemoryStorage {
    fn apply(&mut self, mutation: Mutation) -> Result<MutationOutcome, StorageError> {
        match mutation {
            Mutation::Set { key, value } => {
                self.data.insert(key, value);
                Ok(MutationOutcome::Stored)
            }
            Mutation::Incr { key, by } => {
                self.apply_counter_op(key, |c| c.checked_add(by).ok_or(StorageError::Overflow))
            }
            Mutation::Decr { key, by } => {
                self.apply_counter_op(key, |c| c.checked_sub(by).ok_or(StorageError::Underflow))
            }
            Mutation::Del { key } => {
                self.data.remove(&key);
                Ok(MutationOutcome::Deleted)
            }
        }
    }
    fn read(&self, key: &str) -> Option<StoredValue> {
        self.data.get(key).cloned()
    }
    fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn apply_set_returns_stored() {
        let mut storage = InMemoryStorage::new();
        let result = storage.apply(Mutation::Set {
            key: String::from("foo"),
            value: "bar".into(),
        });
        assert_eq!(result, Ok(MutationOutcome::Stored));
    }

    #[test]
    fn apply_del_returns_deleted() {
        let mut storage = InMemoryStorage::new();
        let result = storage.apply(Mutation::Del {
            key: String::from("foo"),
        });
        assert_eq!(result, Ok(MutationOutcome::Deleted));
    }

    #[test]
    fn apply_incr_on_missing_key_starts_from_zero() {
        let mut storage = InMemoryStorage::new();
        let result = storage.apply(Mutation::Incr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Ok(MutationOutcome::Counter { new_value: 1 }));
    }

    #[test]
    fn apply_incr_on_existing_value_adds_to_it() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: 5.into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Incr {
            key: String::from("foo"),
            by: 3,
        });
        assert_eq!(result, Ok(MutationOutcome::Counter { new_value: 8 }));
        assert_eq!(storage.read("foo"), Some(8.into()));
    }

    #[test]
    fn apply_decr_on_existing_value_subtracts_from_it() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: 5.into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Decr {
            key: String::from("foo"),
            by: 3,
        });
        assert_eq!(result, Ok(MutationOutcome::Counter { new_value: 2 }));
        assert_eq!(storage.read("foo"), Some(2.into()));
    }

    #[test]
    fn apply_incr_on_non_numeric_value_errors_and_preserves_original() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "hello".into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Incr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::NotAnInteger));
        assert_eq!(storage.read("foo"), Some("hello".into()));
    }

    #[test]
    fn apply_decr_on_non_numeric_value_errors_and_preserves_original() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "hello".into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Decr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::NotAnInteger));
        assert_eq!(storage.read("foo"), Some("hello".into()));
    }

    #[test]
    fn apply_incr_overflow_errors_and_preserves_original() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: u64::MAX.into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Incr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::Overflow));
        assert_eq!(storage.read("foo"), Some(u64::MAX.into()));
    }

    #[test]
    fn apply_decr_underflow_errors_and_preserves_original() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: 0.into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Decr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::Underflow));
        assert_eq!(storage.read("foo"), Some(0.into()));
    }

    #[test]
    fn apply_del_removes_existing_value() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "bar".into(),
            })
            .unwrap();
        storage
            .apply(Mutation::Del {
                key: String::from("foo"),
            })
            .unwrap();
        assert!(!storage.exists("foo"));
        assert_eq!(storage.read("foo"), None);
    }

    #[test]
    fn read_returns_none_for_missing_value() {
        let storage = InMemoryStorage::new();
        assert_eq!(storage.read("foo"), None);
    }

    #[test]
    fn read_returns_some_for_existing_value() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "bar".into(),
            })
            .unwrap();
        assert_eq!(storage.read("foo"), Some("bar".into()));
    }

    #[test]
    fn exists_returns_false_for_missing_value() {
        let storage = InMemoryStorage::new();
        assert!(!storage.exists("foo"));
    }

    #[test]
    fn exists_returns_true_for_existing_value() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "bar".into(),
            })
            .unwrap();
        assert!(storage.exists("foo"));
    }
    #[test]
    fn apply_incr_on_numeric_string_errors() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "5".into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Incr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::NotAnInteger));
        assert_eq!(storage.read("foo"), Some("5".into()));
    }

    #[test]
    fn apply_decr_on_numeric_string_errors() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "5".into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Decr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::NotAnInteger));
        assert_eq!(storage.read("foo"), Some("5".into()));
    }

    #[test]
    fn apply_set_can_store_int_variant() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: 42u64.into(),
            })
            .unwrap();
        assert_eq!(storage.read("foo"), Some(StoredValue::Int(42)));
    }

    #[test]
    fn apply_set_can_store_str_variant() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "hello".into(),
            })
            .unwrap();
        assert_eq!(
            storage.read("foo"),
            Some(StoredValue::Str(String::from("hello")))
        );
    }

    #[test]
    fn apply_set_can_overwrite_with_different_variant() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: 42u64.into(),
            })
            .unwrap();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "now a string".into(),
            })
            .unwrap();
        assert_eq!(
            storage.read("foo"),
            Some(StoredValue::Str(String::from("now a string")))
        );
    }

    #[test]
    fn apply_incr_after_overwriting_int_with_str_errors() {
        let mut storage = InMemoryStorage::new();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: 5u64.into(),
            })
            .unwrap();
        storage
            .apply(Mutation::Set {
                key: String::from("foo"),
                value: "oops".into(),
            })
            .unwrap();
        let result = storage.apply(Mutation::Incr {
            key: String::from("foo"),
            by: 1,
        });
        assert_eq!(result, Err(StorageError::NotAnInteger));
    }
}
