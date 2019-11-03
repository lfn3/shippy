use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Project {
    project_id: u64,
    api_token: String,
}

impl Project {
    pub fn get_mr(&self, mr_id: u64) {}
}

#[cfg(test)]
mod tests {
    #[test]
    fn can_get_a_commit() {
        //        "/projects/:id/repository/commits/:sha";
        //        let body = reqwest::get("https://www.rust-lang.org")
        //            .await?
        //            .json()
        //            .await?;
    }
}

#[cfg(all(test, feature = "gitlab_api_tests"))]
mod gitlab_api_tests {
    use crate::git_lab::Project;
    use std::env;

    lazy_static! {
        static ref PROJECT: Project = Project {
            project_id: 15148894,
            api_token: env::var("GITLAB_API_TOKEN").unwrap()
        };
    }

    #[test]
    fn can_get_mr() {
        PROJECT.get_mr(1)
    }
}
