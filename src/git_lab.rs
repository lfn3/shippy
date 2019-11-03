use crate::err::CliError;
use reqwest::Client;
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Deserialize;
use std::fmt::Display;

pub struct Project {
    project_id: u64,
    api_token: String,
    client: reqwest::Client,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct MergeRequest {
    iid: u64,
    title: String,
    description: String,
    author: User,
}

impl Display for MergeRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(self.title.as_str())
            .and(f.write_str(" by "))
            .and(f.write_str(self.author.name.as_str()))
    }
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct User {
    id: u64,
    name: String,
    username: String,
}

impl Project {
    pub fn new(project_id: u64, api_token: String) -> Project {
        Project {
            project_id,
            api_token,
            client: Client::new(),
        }
    }

    pub fn get_mr(&self, mr_id: u64) -> Result<MergeRequest, CliError<'static>> {
        let url = format!(
            "https://gitlab.com/api/v4/projects/{project_id}/merge_requests/{mr_id}",
            project_id = self.project_id,
            mr_id = mr_id
        );

        reqwest::get(url.as_str())
            .and_then(|mut resp| resp.json::<MergeRequest>())
            .map_err(|e| {
                let message = format!("Error getting merge request with id {}", mr_id);
                CliError::Http(message, e)
            })
    }
}

#[cfg(all(test, feature = "gitlab_api_tests"))]
mod gitlab_api_tests {
    use crate::git_lab::Project;
    use std::env;

    lazy_static! {
        static ref PROJECT: Project = Project::new(15148894, env::var("GITLAB_API_TOKEN").unwrap());
    }

    #[test]
    fn can_get_mr() {
        let mr = PROJECT.get_mr(1).unwrap();
        assert_eq!(mr.author.username, "lfn3")
    }
}
