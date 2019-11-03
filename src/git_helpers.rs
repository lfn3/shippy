#[cfg(test)]
pub mod git_helpers {
    use git2::{Commit, Error, Oid, ReferenceType, Repository, Tree};
    use rand::Rng;
    use std::env::temp_dir;
    use std::fs::create_dir;
    use std::iter::FromIterator;
    use std::ops::Add;
    use std::path::PathBuf;

    pub fn tmp_dir() -> PathBuf {
        let mut rng = rand::thread_rng();
        let mut dir = temp_dir();

        dir.push("shippy-test-".to_owned() + rng.gen::<u64>().to_string().as_str());

        let path = dir.as_path();
        create_dir(path).unwrap();
        path.to_owned()
    }

    pub fn initial_commit(repo: &Repository) -> Result<Oid, Error> {
        let sig = repo.signature()?;
        let tree = empty_tree(repo)?;

        repo.commit(Some("HEAD"), &sig, &sig, "Initial Commit", &tree, &[])
    }

    pub fn commit_with_message(repo: &Repository, msg: &str) -> Result<Oid, Error> {
        let sig = repo.signature()?;
        let tree = empty_tree(repo)?;

        match peel_ref(repo, "HEAD") {
            Ok(p) => repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[&p]),
            Err(_) => repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &[]),
        }
    }

    pub fn peel_ref<'repo>(repo: &'repo Repository, name: &str) -> Result<Commit<'repo>, Error> {
        let mut found: Option<Result<Commit, Error>> = Option::None;
        let mut reference = repo.find_reference(name)?;
        while found.is_none() {
            match reference.kind().unwrap() {
                ReferenceType::Direct => found = Option::Some(reference.peel_to_commit()),
                ReferenceType::Symbolic => {
                    let next_name = reference.symbolic_target().unwrap();
                    reference = repo.find_reference(next_name)?
                }
            };
        }

        found.unwrap()
    }

    pub fn empty_commit(repo: &Repository) -> Result<Oid, Error> {
        let sig = repo.signature()?;
        let tree = empty_tree(repo)?;

        let parent = peel_ref(repo, "HEAD")?;

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "An empty commit",
            &tree,
            &[&parent],
        )
    }

    pub fn empty_tree(repo: &Repository) -> Result<Tree, Error> {
        let tree_id = {
            let mut idx = repo.index()?;

            idx.write_tree()?
        };
        repo.find_tree(tree_id)
    }

    pub fn tmp_repo() -> Repository {
        let dir = tmp_dir();
        let repo = Repository::init(dir).unwrap();
        repo
    }

    pub fn lightweight_tag(repo: &Repository, commit_id: Oid, name: &str) -> Result<Oid, Error> {
        let commit_obj = repo.find_object(commit_id, Option::None)?;
        repo.tag_lightweight(name, &commit_obj, false)
    }

    ///This is all more me figuring out how libgit and the rust bindings work than actual tests.
    #[test]
    fn can_create_repo() {
        let dir = tmp_dir();

        Repository::init(dir).unwrap();
    }

    #[test]
    fn can_commit_to_repo() {
        let repo = tmp_repo();

        initial_commit(&repo).unwrap();
    }

    #[test]
    fn can_read_commit_from_repo() {
        let repo = tmp_repo();

        let oid = initial_commit(&repo).unwrap();

        repo.find_commit(oid).unwrap();
    }

    #[test]
    fn can_tag_commit() {
        let repo = tmp_repo();

        let oid = initial_commit(&repo).unwrap();

        let commit = repo.find_commit(oid).unwrap();

        repo.tag(
            "the-tag",
            &commit.into_object(),
            &repo.signature().unwrap(),
            "This is a tag",
            false,
        )
        .unwrap();
    }

    #[test]
    fn can_find_tag() {
        let repo = tmp_repo();

        let commit_id = initial_commit(&repo).unwrap();

        let commit = repo.find_commit(commit_id).unwrap();
        let tag_name = "the-tag";
        let tag_id = repo
            .tag(
                tag_name,
                &commit.into_object(),
                &repo.signature().unwrap(),
                "This is a tag",
                false,
            )
            .unwrap();

        let tags = repo.tag_names(Option::Some("the-*")).unwrap();

        assert_eq!(tags.len(), 1);

        let tag_refname = "refs/tags/".to_owned().add(tags.get(0).unwrap());
        let tag_ref = repo.find_reference(tag_refname.as_str()).unwrap();
    }

    #[test]
    fn can_find_lightweight_tag() {
        let repo = tmp_repo();

        let commit_id = initial_commit(&repo).unwrap();
        lightweight_tag(&repo, commit_id, "the-tag");

        let tags = repo.tag_names(Option::Some("the-*")).unwrap();

        assert_eq!(tags.len(), 1);

        let tag_refname = "refs/tags/".to_owned().add(tags.get(0).unwrap());
        let tag_ref = repo.find_reference(tag_refname.as_str()).unwrap();
    }

    #[test]
    fn can_get_diff_between_a_tag_and_a_given_commit() {
        let repo = tmp_repo();

        let commit_id1 = initial_commit(&repo).unwrap();
        let commit_id2 = empty_commit(&repo).unwrap();
        let commit_id3 = empty_commit(&repo).unwrap();

        let tag_id = lightweight_tag(&repo, commit_id1, "the-tag").unwrap();
        //Interestingly the oid of a lightweight tag is the commit it references?
        let commit = repo.find_commit(tag_id).unwrap();

        let tag_tree = commit.tree().unwrap();
        let commit_tree = repo.find_commit(commit_id3).unwrap().tree().unwrap();

        repo.diff_tree_to_tree(
            Option::Some(&tag_tree),
            Option::Some(&commit_tree),
            Option::None,
        )
        .unwrap();
    }

    #[test]
    fn can_walk_between_a_tag_and_a_given_commit() {
        let repo = tmp_repo();

        let commit_id1 = initial_commit(&repo).unwrap();
        let commit_id2 = empty_commit(&repo).unwrap();
        let commit_id3 = empty_commit(&repo).unwrap();
        let commit_id4 = empty_commit(&repo).unwrap();
        let commit_id5 = empty_commit(&repo).unwrap();

        let tag_id = lightweight_tag(&repo, commit_id1, "the-tag").unwrap();
        let commit = repo.find_commit(tag_id).unwrap();
        let mut revwalk = repo.revwalk().unwrap();

        revwalk.push(commit_id3).unwrap();
        revwalk.hide(tag_id).unwrap();

        let v = Vec::from_iter(revwalk);
        // looks like we don't see the pushed in commit
        //    assert_eq!(v[2].as_ref().unwrap(), &commit_id1);
        assert_eq!(v[1].as_ref().unwrap(), &commit_id2);
        assert_eq!(v[0].as_ref().unwrap(), &commit_id3);
    }
}
