//! Errors returned by config.
//!
//! The primary error returned will be `ParseError`. `ParseError` wraps a number of different
//! types of sub-errors that give more information.
//!
//!
use std::env::VarError;
use std::fmt;
use std::io::Error;
use yaml_rust::scanner::ScanError;
/// Defines a ParseError.
///
/// `ParseError` is a wrapper around several different kinds of sub-errors that may occur. The goal
/// is to give the user what they need without overburdening them with match statements.
///
/// **Examples**
///
/// ```rust
/// use yaml_config::error::ParseError;
/// let error = ParseError { module: "some_mod".to_string(), message: "something broke!".to_string() };
/// ```
#[derive(Debug)]
pub struct ParseError {
    pub module: String,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.module, self.message)
    }
}

impl From<ScanError> for ParseError {
    fn from(error: ScanError) -> Self {
        ParseError {
            module: String::from("yaml_rust::scanner"),
            message: error.to_string(),
        }
    }
}

impl From<VarError> for ParseError {
    fn from(error: VarError) -> Self {
        ParseError {
            module: String::from("std::env"),
            message: error.to_string(),
        }
    }
}

impl From<Error> for ParseError {
    fn from(error: Error) -> Self {
        ParseError {
            module: String::from("std::io"),
            message: error.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ParseError;
    use std::env::VarError;
    use std::io::Error;

    #[test]
    fn test_display_trait() {
        let error = ParseError {
            module: "test::test".to_string(),
            message: "test error".to_string(),
        };
        assert_eq!(format!("{}", error), "test::test: test error")
    }

    // ScanError cant be tested due to private fields.

    #[test]
    fn test_var_error() {
        let error = ParseError::from(VarError::NotPresent);
        assert_eq!(
            format!("{}", error),
            "std::env: environment variable not found"
        );
    }

    #[test]
    fn test_error() {
        let error = ParseError::from(Error::new(std::io::ErrorKind::Unsupported, "bad news"));
        assert_eq!(format!("{}", error), "std::io: bad news");
    }
}
