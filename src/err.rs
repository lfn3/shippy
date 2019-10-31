use std::io;
use std::fmt;

#[derive(Debug)]
pub enum CliError<'this> {
    Str(&'this str),
    String(String),
    Io(&'this str, io::Error),
    Git(&'this str, git2::Error)
}

impl fmt::Display for CliError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Str(msg) => { f.write_str(msg) }
            CliError::String(msg) => { f.write_str(msg) }
            CliError::Io(msg, io_err) => { f.write_str(msg).and(f.write_str(":\n")).and(io_err.fmt(f)) },
            CliError::Git(msg, git_err) => { f.write_str(msg).and(f.write_str(":\n")).and(git_err.fmt(f)) },
        }
    }
}

impl std::error::Error for CliError<'_> {

}