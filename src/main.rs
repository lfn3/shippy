use std::error::Error;

use git2::Repository;
use std::fmt::{Debug, Display, Formatter};
use std::borrow::Borrow;
use std::{fmt, env};

mod git_helpers;

fn main() -> Result<(), SimpleError<'static>> {
    let cwd = env::current_dir().map_err(| e |
        SimpleError::new_with_cause("Could not get current_dir".to_owned(), e.borrow()))?;
    let args : Vec<String> = env::args().collect();

    Ok(())
}

#[derive(Debug)]
struct  SimpleError<'cause> {
    message: String,
    cause: Option<&'cause dyn Error>
}

impl SimpleError<'_> {
    pub fn new(message: String) -> SimpleError<'static> {
        SimpleError { message, cause: None }
    }

    pub fn new_with_cause(message: String, cause: &'_ dyn Error) -> SimpleError<'_> {
        SimpleError { message, cause: Option::Some(cause) }
    }
}

impl Display for SimpleError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.cause {
            None => { f.write_str(self.message.as_str()) },
            Some(_) => { f.write_str(self.message.as_str()).and(self.cause.fmt(f)) },
        }

    }
}

impl Error for SimpleError<'_> {

}

fn find_greatest_tag(repo: &Repository, prefix: &str) -> Result<String, SimpleError<'static>> {
    let mut search_string = prefix.to_owned();
    search_string.push('*');
    //TODO: don't unwrap here
    let tags = repo.tag_names(Option::Some(search_string.borrow())).unwrap();

    let mut max : Option<u64> = Option::None;
    //TODO: what's the deal with None options here?
    for tag in tags.iter().filter(Option::is_some).map(Option::unwrap) {
        let (_, suffix) : (&str, &str) = tag.split_at(prefix.len());
        let parsed = suffix.parse::<u64>()
            .map_err(|_| SimpleError::new(format!("Could not parse u64 from: {}, in tag: {}", suffix, tag)))?;

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
        Result::Err(SimpleError::new(message))
    }
}

#[cfg(test)]
mod tests {
    use crate::git_helpers::git_helpers::{tmp_repo, initial_commit, lightweight_tag, empty_commit};
    use crate::{find_greatest_tag};

    #[test]
    fn find_greatest_tag_returns_error_for_empty_repo() {
        let repo = tmp_repo();

        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap_err().message,
                   "Could not find any tags with prefix: tag-");
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

        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap_err().message,
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