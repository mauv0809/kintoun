use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Command {
    Set { key: String, value: String },
    Get { key: String },
    Del { key: String },
    Exists { key: String },
    Incr { key: String, by: u64 },
    Decr { key: String, by: u64 },
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("empty input")]
    Empty,
    #[error("missing argument: {0}")]
    MissingArg(&'static str),
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("invalid amount {input:?}: {reason}")]
    InvalidAmount { input: String, reason: String },
    #[error("too many arguments: {0}")]
    TooManyArgs(&'static str),
}

// Helper functions
fn expect_done<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    command: &'static str,
) -> Result<(), ParseError> {
    match parts.next() {
        None => Ok(()),
        Some(_) => Err(ParseError::TooManyArgs(command)),
    }
}

fn take_arg<'a>(
    parts: &mut impl Iterator<Item = &'a str>,
    label: &'static str,
) -> Result<String, ParseError> {
    parts
        .next()
        .ok_or(ParseError::MissingArg(label))
        .map(str::to_string)
}

// Pull an optional amount; defaults to 1 if absent.
fn take_amount<'a>(parts: &mut impl Iterator<Item = &'a str>) -> Result<u64, ParseError> {
    match parts.next() {
        None => Ok(1),
        Some(s) => s.parse::<u64>().map_err(|e| ParseError::InvalidAmount {
            input: s.to_string(),
            reason: e.to_string(),
        }),
    }
}
impl FromStr for Command {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts = input.split_whitespace();
        let verb = parts.next().ok_or(ParseError::Empty)?;
        match verb.to_ascii_lowercase().as_str() {
            "set" => {
                let key: String = take_arg(&mut parts, "key")?;
                let value = take_arg(&mut parts, "value")?;
                expect_done(&mut parts, "set")?;
                Ok(Command::Set { key, value })
            }
            "get" => {
                let key: String = take_arg(&mut parts, "key")?;
                expect_done(&mut parts, "get")?;
                Ok(Command::Get { key })
            }
            "del" => {
                let key: String = take_arg(&mut parts, "key")?;
                expect_done(&mut parts, "del")?;
                Ok(Command::Del { key })
            }
            "exists" => {
                let key: String = take_arg(&mut parts, "key")?;
                expect_done(&mut parts, "exists")?;
                Ok(Command::Exists { key })
            }
            "incr" => {
                let key: String = take_arg(&mut parts, "key")?;
                let by: u64 = take_amount(&mut parts)?;
                expect_done(&mut parts, "incr")?;
                Ok(Command::Incr { key, by })
            }
            "decr" => {
                let key: String = take_arg(&mut parts, "key")?;
                let by: u64 = take_amount(&mut parts)?;
                expect_done(&mut parts, "decr")?;
                Ok(Command::Decr { key, by })
            }
            _ => Err(ParseError::UnknownCommand(verb.to_string())),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_set_with_two_args_returns_set_command() {
        let input = "set foo bar";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Set {
                key: String::from("foo"),
                value: String::from("bar")
            }
        );
    }
    #[test]
    fn parse_set_without_value_returns_missing_arg_error() {
        let input = "set foo";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::MissingArg(s)) => assert_eq!(s, "value"),
            other => panic!("got {other:?}"),
        };
    }
    #[test]
    fn parse_set_without_key_returns_missing_arg_error() {
        let input = "set";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::MissingArg(s)) => assert_eq!(s, "key"),
            other => panic!("got {other:?}"),
        };
    }
    #[test]
    fn parse_get_with_one_arg_returns_get_command() {
        let input = "get foo";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Get {
                key: String::from("foo")
            }
        );
    }
    #[test]
    fn parse_del_with_one_arg_returns_del_command() {
        let input = "del foo";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Del {
                key: String::from("foo")
            }
        );
    }
    #[test]
    fn parse_exists_with_one_arg_returns_exists_command() {
        let input = "exists foo";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Exists {
                key: String::from("foo")
            }
        );
    }

    #[test]
    fn parse_empty_string_returns_empty_error() {
        let input = "";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::Empty) => {}
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parse_unknown_verb_returns_unknown_command_error() {
        let input = "foosball bar";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::UnknownCommand(s)) => assert_eq!(s, "foosball"),
            other => panic!("got {other:?}"),
        }
    }
    #[test]
    fn parse_unknown_verb_returns_unknown_command_error_keeping_case() {
        let input = "Foosball bar";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::UnknownCommand(s)) => assert_eq!(s, "Foosball"),
            other => panic!("got {other:?}"),
        }
    }
    #[test]
    fn parse_incr_without_amount_defaults_by_to_one() {
        let input = "incr counter";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 1
            }
        );
    }
    #[test]
    fn parse_incr_with_amount_returns_incr_command() {
        let input = "incr counter 10";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn parse_incr_with_invalid_amount_returns_invalid_amount_error() {
        let input = "incr counter foo";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::InvalidAmount { input, reason }) => {
                assert_eq!(input, "foo");
                assert!(
                    reason.contains("invalid digit"),
                    "unexpected reason: {reason:?}"
                );
            }
            other => panic!("got {other:?}"),
        }
    }
    #[test]
    fn parse_decr_without_amount_defaults_by_to_one() {
        let input = "decr counter";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Decr {
                key: String::from("counter"),
                by: 1
            }
        );
    }
    #[test]
    fn parse_decr_with_amount_returns_decr_command() {
        let input = "decr counter 10";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Decr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn parse_decr_with_invalid_amount_returns_invalid_amount_error() {
        let input = "decr counter foo";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::InvalidAmount { input, reason }) => {
                assert_eq!(input, "foo");
                assert!(
                    reason.contains("invalid digit"),
                    "unexpected reason: {reason:?}"
                );
            }
            other => panic!("got {other:?}"),
        }
    }
    #[test]
    fn parse_uppercase_verb_to_lowercase() {
        let input = "DECR counter 10";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Decr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn parse_mixed_case_verb_to_lowercase() {
        let input = "DeCr counter 10";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Decr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn leading_whitespace_is_ignored() {
        let input = " incr counter 10";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn trailing_whitespace_is_ignored() {
        let input = "incr counter 10 ";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn multiple_whitespace_is_ignored() {
        let input = "  incr  counter  10  ";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn tabs_are_ignored() {
        let input = "incr\tcounter\t10";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn newline_is_ignored() {
        let input = "incr\ncounter\n10\n";
        let cmd: Command = input.parse().unwrap();
        assert_eq!(
            cmd,
            Command::Incr {
                key: String::from("counter"),
                by: 10
            }
        );
    }
    #[test]
    fn all_whitespace_input_returns_empty_error() {
        let input = "   ";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::Empty) => {}
            other => panic!("got {other:?}"),
        }
    }
    #[test]
    fn parse_get_with_extra_arg_returns_too_many_args_error() {
        let input = "get foo bar";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::TooManyArgs(command)) => assert_eq!(command, "get"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parse_set_with_extra_arg_returns_too_many_args_error() {
        let input = "set foo bar baz";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::TooManyArgs(command)) => assert_eq!(command, "set"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parse_del_with_extra_arg_returns_too_many_args_error() {
        let input = "del foo bar";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::TooManyArgs(command)) => assert_eq!(command, "del"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parse_exists_with_extra_arg_returns_too_many_args_error() {
        let input = "exists foo bar";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::TooManyArgs(command)) => assert_eq!(command, "exists"),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn parse_incr_with_extra_arg_returns_too_many_args_error() {
        let input = "incr counter 5 extra";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::TooManyArgs(command)) => assert_eq!(command, "incr"),
            other => panic!("got {other:?}"),
        }
    }
    #[test]
    fn parse_decr_with_extra_arg_returns_too_many_args_error() {
        let input = "decr counter 5 extra";
        let result: Result<Command, ParseError> = input.parse();
        match result {
            Err(ParseError::TooManyArgs(command)) => assert_eq!(command, "decr"),
            other => panic!("got {other:?}"),
        }
    }
}
