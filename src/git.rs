use git2::{Repository, Oid, Commit, BranchType};
use std::borrow::Borrow;
use crate::err::CliError;

fn commits_between_refs<'repo>(repo: &'repo Repository, to: &str, from: &str) -> Result<Vec<Commit<'repo>>, CliError<'static>> {
    let to_oid = find_commit_oid(repo, to)?;
    let from_oid = find_commit_oid(repo, from)?;

    commits_between_oids(repo, to_oid, from_oid)
}

fn find_commit_oid(repo: &Repository, s: &str) -> Result<Oid, CliError<'static>> {
    find_commit_oid_via_ref(repo, s)
        .or_else(| _ | find_commit_oid_via_tag_name(repo, s))
        .or_else(| _ | find_commit_oid_via_branch_name(repo, s))
}

fn find_commit_oid_via_branch_name(repo: &Repository, branch_name: &str) -> Result<Oid, CliError<'static>> {
    repo.find_branch(branch_name, BranchType::Local)
        .and_then(| b | b.get().peel_to_commit().map(| c | c.id()))
        .map_err(| e | CliError::Git("Could not find branch", e))
}

fn find_commit_oid_via_tag_name(repo: &Repository, tag_name: &str) -> Result<Oid, CliError<'static>> {
    find_commit_oid_via_ref(repo, format!("refs/tags/{}", tag_name).as_str())
}

fn find_commit_oid_via_ref(repo: &Repository, git_ref: &str) -> Result<Oid, CliError<'static>> {
    repo.find_reference(git_ref)
        .and_then(|r| r.peel_to_commit())
        .map(|c| c.id())
        .map_err(|e| CliError::Git("Could not parse ref", e))
}

/// from is exclusive
fn commits_between_oids(repo: &Repository, to: Oid, from: Oid) -> Result<Vec<Commit>, CliError<'static>> {
    let mut revwalk = repo.revwalk().map_err(|e| CliError::Git("Could not create revwalk", e))?;

    revwalk.push(to).map_err(|e| CliError::Git("Could not find commit", e))?;
    revwalk.hide(from).map_err(|e| CliError::Git("Could not find commit", e))?;
    let mut v = Vec::new();
    for rev in revwalk {
        let oid = rev.map_err(|e| CliError::Git("Error during revwalk", e))?;
        let commit = repo.find_commit(oid).map_err(|e| CliError::Git("Could not find commit", e))?;
        v.push(commit)
    }

    Ok(v)
}

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

    if let Some(max) = max {
        search_string.pop();
        search_string.push_str(max.to_string().as_str());
        Result::Ok(search_string)
    } else {
        let message = format!("Could not find any tags with prefix: {}", prefix);
        Result::Err(CliError::String(message))
    }
}

#[cfg(test)]
mod tests {
    use crate::git_helpers::git_helpers::{tmp_repo, initial_commit, lightweight_tag, empty_commit};
    use crate::git::{find_greatest_tag, commits_between_oids, find_commit_oid};


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
    fn find_greatest_tag_returns_tag_with_highest_number() {
        let repo = tmp_repo();

        let initial_commit = initial_commit(&repo).unwrap();
        lightweight_tag(&repo, initial_commit, "tag-123");

        let commit_2 = empty_commit(&repo).unwrap();
        lightweight_tag(&repo, commit_2, "tag-2");
        assert_eq!(find_greatest_tag(&repo, "tag-").unwrap(), "tag-123");
    }

    #[test]
    fn commits_between_gets_no_commits_since_from_is_exclusive() {
        let repo = tmp_repo();
        let initial_commit = initial_commit(&repo).unwrap();

        let result = commits_between_oids(&repo, initial_commit, initial_commit).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn commits_between_gets_single_commit() {
        let repo = &tmp_repo();
        let initial_commit = initial_commit(repo).unwrap();
        let commit_2 = empty_commit(repo).unwrap();

        let result = commits_between_oids(repo, commit_2, initial_commit).unwrap();

        assert_eq!(result.len(), 1);
        let c = &result[0];
        assert_eq!(c.id(), commit_2);
    }

    #[test]
    fn commits_between_gets_several_commits_in_order() {
        let repo = &tmp_repo();
        let initial_commit = initial_commit(repo).unwrap();
        let commit_2 = empty_commit(repo).unwrap();
        let commit_3 = empty_commit(repo).unwrap();
        let commit_4 = empty_commit(repo).unwrap();

        let result = commits_between_oids(repo, commit_4, initial_commit).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[2].id(), commit_2);
        assert_eq!(result[1].id(), commit_3);
        assert_eq!(result[0].id(), commit_4);
    }

    #[test]
    fn can_find_commit_from_ref() {
        let repo = &tmp_repo();
        let initial_commit = initial_commit(repo).unwrap();

        assert_eq!(initial_commit, find_commit_oid(repo, "HEAD").unwrap());
    }

    #[test]
    fn can_find_commit_from_tag() {
        let repo = &tmp_repo();
        let initial_commit = initial_commit(repo).unwrap();
        lightweight_tag(&repo, initial_commit, "tag-123");

        assert_eq!(initial_commit, find_commit_oid(repo, "tag-123").unwrap());
    }

    #[test]
    fn can_find_commit_from_branch() {
        let repo = &tmp_repo();
        let initial_commit = initial_commit(repo).unwrap();
        let c = repo.find_commit(initial_commit).unwrap();
        repo.branch("a-branch", &c, false);

        assert_eq!(initial_commit, find_commit_oid(repo, "a-branch").unwrap());
    }
}