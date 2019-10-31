use std::error::Error;

use git2::Repository;
use std::fmt::{Debug, Display, Formatter};
use std::borrow::Borrow;
use std::fmt;
use crate::err::CliError;

mod git_helpers;
pub mod err;

fn find_greatest_tag(repo: &Repository, prefix: &str) -> Result<String, CliError<'static>> {
    if prefix.is_empty() {
        return Result::Err(CliError::Str("Can't find greatest tag with no prefix, would find all tags."))
    }
    let mut search_string = prefix.to_owned();
    search_string.push('*');
    let tags = repo.tag_names(Option::Some(search_string.borrow()))
        .map_err(|e| CliError::Git("Could not read tags from repo", e))?;

    let mut max : Option<u64> = Option::None;
    //TODO: what's the deal with None options here?
    for tag in tags.iter().filter(Option::is_some).map(Option::unwrap) {
        let (_, suffix) : (&str, &str) = tag.split_at(prefix.len());
        let parsed = suffix.parse::<u64>()
            .map_err(|_| CliError::String(format!("Could not parse u64 from: {}, in tag: {}", suffix, tag)))?;

        if max.is_none() || max.unwrap() < parsed {
            max = Some(parsed);
        }
    }

    if max.is_some() {
        search_string.pop();
        search_string.push_str(max.unwrap().to_string().as_str());
        Result::Ok(search_string)
    } else {
        let message = format!("Could not find any tags with prefix: {}", prefix);
        Result::Err(CliError::String(message))
    }
}

#[cfg(test)]
mod tests {
    use crate::git_helpers::git_helpers::{tmp_repo, initial_commit, lightweight_tag, empty_commit};
    use crate::{find_greatest_tag};

    #[test]
    fn find_greatest_tag_returns_error_for_empty_repo() {
        let repo = tmp_repo();

        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap_err().to_string(),
                   "Could not find any tags with prefix: tag-");
    }

    #[test]
    fn find_greatest_tag_returns_error_for_empty_prefix() {
        let repo = tmp_repo();

        assert_eq!(find_greatest_tag(&repo, "").unwrap_err().to_string(),
                   "Can't find greatest tag with no prefix, would find all tags.");
    }

    #[test]
    fn find_greatest_tag_returns_single_tag_with_matching_prefix() {
        let repo = tmp_repo();

        let initial_commit = initial_commit(&repo).unwrap();
        lightweight_tag(&repo, initial_commit, "tag-123");

        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap(), "tag-123");
    }

    #[test]
    fn find_greatest_tag_returns_error_for_tag_with_non_numeric_suffix() {
        let repo = tmp_repo();

        let initial_commit = initial_commit(&repo).unwrap();
        lightweight_tag(&repo, initial_commit, "tag-abc");

        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap_err().to_string(),
                   "Could not parse u64 from: abc, in tag: tag-abc");
    }

    #[test]
    fn test_find_greatest_tag_returns_tag_with_highest_number() {
        let repo = tmp_repo();

        let initial_commit = initial_commit(&repo).unwrap();
        lightweight_tag(&repo, initial_commit, "tag-123");

        let commit_2 = empty_commit(&repo).unwrap();
        lightweight_tag(&repo, commit_2, "tag-2");
        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap(), "tag-123");
    }
}