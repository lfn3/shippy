pub mod err;
mod git;
mod git_helpers;
mod git_lab;
use serde::Deserialize;

#[macro_use]
extern crate lazy_static;

use crate::err::CliError;
use git2::{Commit, Repository};

#[derive(Debug, PartialEq, Deserialize)]
struct Config {
    project_id: u64,
    api_token: ApiToken,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "from")]
pub enum ApiToken {
    EnvVar { name: String },
}

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

#[cfg(test)]
mod tests {
    use crate::ApiToken::EnvVar;
    use crate::Config;

    #[test]
    fn can_deserialize_config_yaml() {
        let yaml_str = r#"
            project_id: 1234
            api_token:
                from: EnvVar
                name: API_TOKEN
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
