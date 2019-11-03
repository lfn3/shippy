pub mod err;
mod git;
mod git_helpers;
mod git_lab;

#[macro_use]
extern crate lazy_static;

use crate::err::CliError;
use git2::{Commit, Repository};

pub fn print_release_notes(
    repo: &Repository,
    tag_prefix: &str,
    up_to: &str,
) -> Result<(), CliError<'static>> {
    let max_tag = git::find_greatest_tag(repo, tag_prefix)?;
    println!("Searching between {} and {}", max_tag, up_to);

    let commits = git::commits_between_refs(repo, up_to, max_tag.as_str())?;
    print!("Found {} commits", commits.len());

    let mrs: Vec<u64> = commits
        .into_iter()
        .map(|c: Commit| git::associated_mr(&c))
        .filter(|o| o.is_some())
        .map(|o| o.unwrap())
        .collect();
    println!(", pointing to {} merge requests:", mrs.len());

    Ok(())
}
