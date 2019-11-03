use git2::Repository;
use shippy::err::CliError;
use shippy::print_release_notes;
use std::env;

extern crate shippy;

fn main() -> Result<(), CliError<'static>> {
    let cwd = env::current_dir().map_err(|e| CliError::Io("Could not get current_dir", e))?;
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(CliError::Str(
            "Expected at least one argument containing a tag prefix",
        ));
    }

    let repo = &Repository::open(cwd).map_err(|e| CliError::Git("Could not open repository", e))?;

    print_release_notes(
        repo,
        args[1].as_str(),
        args.get(1).map_or_else(|| "HEAD", |s| s.as_str()),
    )
}
