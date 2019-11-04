use git2::Repository;
use shippy::err::CliError;
use shippy::git_lab::Project;
use shippy::{print_release_notes, Config};
use std::env;
use std::fs::File;
use structopt::StructOpt;
use std::path::PathBuf;

extern crate shippy;

#[derive(StructOpt)]
#[structopt(name = "shippy", about = "Release note generator")]
struct Opts {
    #[structopt(index = 1)]
    tag_prefix: String,

    #[structopt(index = 2, default_value = "HEAD")]
    up_to: String,

    #[structopt(short = "c", long = "config_file", parse(from_os_str), default_value = "./shippy.yml")]
    config_file: PathBuf,

    #[structopt(short = "t", long = "team")]
    team: Option<String>
}

fn main() -> Result<(), CliError<'static>> {
    let cwd = env::current_dir().map_err(|e| CliError::Io("Could not get current_dir", e))?;
    let opts = Opts::from_args();

    let cfg_file =
        File::open(opts.config_file).map_err(|e| CliError::Io("Could not open config file", e))?;

    let cfg: Config = serde_yaml::from_reader(cfg_file)
        .map_err(|e| CliError::Yaml("Could not deserialize config file", e))?;

    let proj = Project::new(cfg.base_url, cfg.project_id, cfg.api_token.get()?);

    let repo = &Repository::open(cwd).map_err(|e| CliError::Git("Could not open repository", e))?;

    print_release_notes(
        &proj,
        repo,
        opts.tag_prefix.as_str(),
        opts.up_to.as_str(),
    )
}
