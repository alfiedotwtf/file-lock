use std::str::FromStr;
use std::fmt;
use std::error::Error;

/// Represents the kind of lock (e.g. *blocking*, *non-blocking*)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Attempt a lock without blocking the call
    NonBlocking,
    /// Attempt a lock and return from the call once the lock was obtained.
    Blocking,
}

/// Represents a file access mode, e.g. read or write
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Mode {
    /// Use this to obtain a shared lock, i.e. there may be any amount of readers
    /// at the same time.
    Read,
    /// Used to indicate an exclusive lock, i.e. there may only be one writer at a time.
    Write
}

impl AsRef<str> for Kind {
    fn as_ref(&self) -> &str {
        match *self {
            Kind::NonBlocking => "nowait",
            Kind::Blocking => "wait",
        }
    }
}

impl FromStr for Kind {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "nowait" => Ok(Kind::NonBlocking),
            "wait" => Ok(Kind::Blocking),
            _ => Err(ParseError(format!("Unknown Kind: {}", input))),
        }
    }
}

impl AsRef<str> for Mode {
    fn as_ref(&self) -> &str {
        match *self {
            Mode::Read => "read",
            Mode::Write => "write",
        }
    }
}

impl FromStr for Mode {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "read" => Ok(Mode::Read),
            "write" => Ok(Mode::Write),
            _ => Err(ParseError(format!("Unknown Mode: {}", input)))
        }
    }
}

impl Into<i32> for Mode {
    fn into(self) -> i32 {
        match self {
            Mode::Read => 0,
            Mode::Write => 1,
        }
    }
}

impl Into<i32> for Kind {
    fn into(self) -> i32 {
        match self {
            Kind::NonBlocking => 0,
            Kind::Blocking => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(String);

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}
