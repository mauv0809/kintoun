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

#[derive(Debug, PartialEq)]
pub enum ParseError {
    Empty,
    UnknownCommand(String),
}
impl FromStr for Command {
    type Err = ParseError;
    fn from_str(_input: &str) -> Result<Self, Self::Err> {
        todo!("write me!");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_set_with_two_args_returns_set_command() {
        let input = "set foo bar";
        let parsed: Result<Command, ParseError> = input.parse();
        let cmd = parsed.unwrap();
        assert_eq!(
            cmd,
            Command::Set {
                key: String::from("foo"),
                value: String::from("bar")
            }
        );
    }
    #[test]
    fn parse_get_with_one_arg_returns_get_command() {
        // TODO: input = "get foo"
        // TODO: parse → unwrap → assert_eq! against Command::Get { key: "foo".into() }
        //   ("foo".into()  is the same as "foo".to_string() here — common shorthand)
        todo!("write me"); // todo!() panics with this message at runtime
    }

    #[test]
    fn parse_empty_string_returns_empty_error() {
        // TODO: input = ""
        // TODO: parse → assert it's Err(ParseError::Empty)
        // Hint — for asserting *which* error variant, two options:
        //   (1) assert!(matches!(result, Err(ParseError::Empty)));
        //   (2) match result { Err(ParseError::Empty) => {}, other => panic!("got {other:?}") }
        // (1) is concise; (2) gives a better failure message. Either is fine.
        todo!("write me");
    }

    #[test]
    fn parse_unknown_verb_returns_unknown_command_error() {
        // TODO: input = "frobnicate foo"
        // TODO: parse → assert it's Err(ParseError::UnknownCommand("frobnicate".into()))
        // (Assumes ParseError::UnknownCommand carries the offending verb as a String —
        //  that's a design choice you'll make when you stub ParseError.)
        todo!("write me");
    }
    #[test]
    fn parse_incr_without_amount_defaults_by_to_one() {
        // TODO: input = "incr counter"
        // TODO: parse → unwrap → assert_eq! against Command::Incr { key: "counter".into(), by: 1 }
        todo!("write me");
    }
}
