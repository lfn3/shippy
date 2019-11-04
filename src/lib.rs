pub mod err;
mod git;
mod git_helpers;
pub mod git_lab;
use serde::Deserialize;

#[macro_use]
extern crate lazy_static;

use crate::err::CliError;
use crate::git_lab::Project;
use git2::{Commit, Repository};
use std::collections::HashMap;
use std::env;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub base_url: String,
    pub project_id: u64,
    pub api_token: ApiToken,
    pub teams: HashMap<String, Vec<String>>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "from")]
pub enum ApiToken {
    EnvVar { name: String },
}

impl ApiToken {
    pub fn get(&self) -> Result<String, CliError<'static>> {
        match self {
            ApiToken::EnvVar { name } => env::var(name).map_err(|_err| {
                CliError::String(format!(
                    "Could not find api token in environment variable {}",
                    name
                ))
            }),
        }
    }
}

pub fn print_release_notes(
    proj: &Project,
    repo: &Repository,
    tag_prefix: &str,
    up_to: &str,
) -> Result<(), CliError<'static>> {
    let max_tag = git::find_greatest_tag(repo, tag_prefix)?;
    println!("Searching between {} and {}", max_tag, up_to);

    let commits = git::commits_between_refs(repo, up_to, max_tag.as_str())?;
    print!("Found {} commits", commits.len());

    let mr_ids: Vec<u64> = commits
        .into_iter()
        .map(|c: Commit| git::associated_mr(&c))
        .filter(|o| o.is_some())
        .map(|o| o.unwrap())
        .collect();
    println!(", pointing to {} merge requests:", mr_ids.len());

    let mrs = proj.get_mrs(mr_ids)?;

    for mr in mrs {
        println!("{}", mr)
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::ApiToken::EnvVar;
    use crate::Config;

    #[test]
    fn can_deserialize_config_yaml() {
        let yaml_str = r#"
            base_url: "https://gitlab.com"
            project_id: 1234
            api_token:
                from: EnvVar
                name: API_TOKEN
            teams:
                A:
                    - Alice
        "#;
        let cfg: Config = serde_yaml::from_str(yaml_str).unwrap();
        assert_eq!(cfg.project_id, 1234);
        assert_eq!(
            cfg.api_token,
            EnvVar {
                name: "API_TOKEN".to_string()
            }
        );
    }
}
