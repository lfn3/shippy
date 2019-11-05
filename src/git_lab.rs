use crate::err::CliError;
use reqwest::{Client, Response};
use serde::export::fmt::Error;
use serde::export::Formatter;
use serde::Deserialize;
use std::fmt::Display;
use std::collections::HashSet;
use std::iter::FromIterator;

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

    fn get(&self, url : String) -> Result<Response, CliError<'static>> {
        let req = self.client
            .get(url.as_str())
            .header("Private-Token", self.api_token.clone())
            .build()
            .map_err(|e| {
                let message = format!(
                    "Could not build request for {}",
                    url
                );
                CliError::Http(message, e)
            })?;

        self.client.execute(req).map_err(|e| {
            CliError::Http(format!("Error getting {}", url), e)
        })
    }

    pub fn get_mrs(&self, mr_ids: Vec<u64>) -> Result<Vec<MergeRequest>, CliError<'static>> {
        let mut left_to_find : HashSet<u64> = HashSet::from_iter(mr_ids.iter().cloned());
        let mut result : Vec<MergeRequest> = Vec::new();
        let mut page = 0;

        while !left_to_find.is_empty() {
            let url = format!(
                "{base_url}/api/v4/projects/{project_id}/merge_requests?page={page}",
                base_url = self.base_url,
                project_id = self.project_id,
                page = page
            );

            let mut response = self.get(url.clone())?;

            let mrs = response.json::<Vec<MergeRequest>>().map_err(|e| {
                let message = format!(
                    "Could not deserialize json from {}:\n {:#?}",
                    url, response
                );
                CliError::Http(message, e)
            })?;

            if mrs.is_empty() {
                let msg = format!("Could not find mrs with ids {:#?}", left_to_find);
                return Err(CliError::String(msg))
            }

            for mr in mrs {
                if left_to_find.remove(&mr.iid) {
                    result.push(mr)
                }
            }

            page = page + 1
        }

        Ok(result)
    }

    pub fn get_mr(&self, mr_id: u64) -> Result<MergeRequest, CliError<'static>> {
        let url = format!(
            "{base_url}/api/v4/projects/{project_id}/merge_requests/{mr_id}",
            base_url = self.base_url,
            project_id = self.project_id,
            mr_id = mr_id
        );

        let mut response = self.get(url.clone())?;

        response.json::<MergeRequest>().map_err(|e| {
            let message = format!(
                "Could not deserialize json from {}:\n {:#?}",
                url, response
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
        let mr_ids = vec!(2, 1);
        assert_eq!(PROJECT.get_mrs(mr_ids.clone()).unwrap(), naive_get_mrs(&PROJECT, mr_ids).unwrap())
    }
}
