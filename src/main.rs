use git2::Repository;
use shippy::err::CliError;
use shippy::git_lab::Project;
use shippy::{print_release_notes, Config};
use std::env;
use std::fs::File;

extern crate shippy;

fn main() -> Result<(), CliError<'static>> {
    let cwd = env::current_dir().map_err(|e| CliError::Io("Could not get current_dir", e))?;
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err(CliError::Str(
            "Expected at least one argument containing a tag prefix",
        ));
    }

    let mut cfg_path = cwd.clone();
    cfg_path.push("shippy.yml");
    let cfg_file =
        File::open(cfg_path).map_err(|e| CliError::Io("Could not open config file", e))?;

    let cfg: Config = serde_yaml::from_reader(cfg_file)
        .map_err(|e| CliError::Yaml("Could not deserialize config file", e))?;

    let proj = Project::new(cfg.base_url, cfg.project_id, cfg.api_token.get()?);

    let repo = &Repository::open(cwd).map_err(|e| CliError::Git("Could not open repository", e))?;

    print_release_notes(
        &proj,
        repo,
        args[1].as_str(),
        args.get(2).map_or_else(|| "HEAD", |s| s.as_str()),
    )
}
