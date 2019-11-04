use crate::err::CliError;
use reqwest::Client;
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Deserialize;
use std::fmt::Display;

pub struct Project {
    base_url: String,
    project_id: u64,
    api_token: String,
    client: reqwest::Client,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct MergeRequest {
    iid: u64,
    title: String,
    description: String,
    pub author: User,
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
    pub username: String,
}

impl Project {
    pub fn new(base_url: String, project_id: u64, api_token: String) -> Project {
        Project {
            base_url,
            project_id,
            api_token,
            client: Client::new(),
        }
    }

    //TODO: use the list endpoint instead
    pub fn get_mrs(&self, mr_ids: Vec<u64>) -> Result<Vec<MergeRequest>, CliError<'static>> {
        mr_ids.iter().map(|id| self.get_mr(*id)).collect()
    }

    pub fn get_mr(&self, mr_id: u64) -> Result<MergeRequest, CliError<'static>> {
        let url = format!(
            "{base_url}/api/v4/projects/{project_id}/merge_requests/{mr_id}",
            base_url = self.base_url,
            project_id = self.project_id,
            mr_id = mr_id
        );
        let req = self
            .client
            .get(url.as_str())
            .header("Private-Token", self.api_token.clone())
            .build()
            .map_err(|e| {
                let message = format!(
                    "Could not build request for merge request with id {}",
                    mr_id
                );
                CliError::Http(message, e)
            })?;

        let mut response = self.client.execute(req).map_err(|e| {
            CliError::Http(format!("Error getting merge request with id {}", mr_id), e)
        })?;

        response.json::<MergeRequest>().map_err(|e| {
            let message = format!(
                "Could not deserialize json for merge request with id {} from:\n {:#?}",
                mr_id, response
            );
            CliError::Http(message, e)
        })
    }
}

#[cfg(all(test, feature = "gitlab_api_tests"))]
mod gitlab_api_tests {
    use crate::git_lab::{Project, MergeRequest};
    use std::env;
    use crate::err::CliError;

    lazy_static! {
        static ref PROJECT: Project = Project::new(
            "http://gitlab.com".to_string(),
            15148894,
            env::var("GITLAB_API_TOKEN").unwrap()
        );
    }

    pub fn naive_get_mrs(project : &Project, mr_ids: Vec<u64>) -> Result<Vec<MergeRequest>, CliError<'static>> {
        mr_ids.iter().map(|id| project.get_mr(*id)).collect()
    }

    #[test]
    fn can_get_mr() {
        let mr = PROJECT.get_mr(1).unwrap();
        assert_eq!(mr.author.username, "lfn3")
    }

    #[test]
    fn can_get_mrs() {
        let mr_ids = vec!(1, 2);
        assert_eq!(PROJECT.get_mrs(mr_ids.clone()).unwrap(), naive_get_mrs(&PROJECT, mr_ids).unwrap())
    }
}
