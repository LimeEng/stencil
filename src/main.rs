use clap::{crate_version, App, Arg};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
enum ProjectType {
    Executable,
    Library,
}

#[derive(Debug, Clone)]
enum PublishOptions {
    All,
    None,
    Crates,
    Binaries,
}

const CI_WORKFLOW: &str = include_str!("../resources/ci.yml");
const RELEASE_ALL_WORKFLOW: &str = include_str!("../resources/release_all.yml");
const RELEASE_CRATES_WORKFLOW: &str = include_str!("../resources/release_crates.yml");
const RELEASE_BINARIES_WORKFLOW: &str = include_str!("../resources/release_binaries.yml");

fn main() {
    let app = App::new("stencil")
        .version(crate_version!())
        .arg(
            Arg::new("name")
                .value_name("Name of project")
                .about("Sets the name for the project")
                .takes_value(false)
                .required(true),
        )
        .arg(
            Arg::new("exe")
                .about("Determines what type of project to generate")
                .long("exe")
                .required_unless_present("lib")
                .conflicts_with("lib"),
        )
        .arg(
            Arg::new("lib")
                .about("Determines what type of project to generate")
                .long("lib")
                .required_unless_present("exe")
                .conflicts_with("exe"),
        )
        .arg(
            Arg::new("all")
                .about("Determines what publishing options to use")
                .long("all")
                .required_unless_present_any(&["binaries", "crates", "none"])
                .conflicts_with_all(&["binaries", "crates", "none"]),
        )
        .arg(
            Arg::new("none")
                .about("Determines what publishing options to use")
                .long("none")
                .required_unless_present_any(&["crates", "binaries", "all"])
                .conflicts_with_all(&["crates", "binaries", "all"]),
        )
        .arg(
            Arg::new("crates")
                .about("Determines what publishing options to use")
                .long("crates")
                .required_unless_present_any(&["binaries", "all", "none"])
                .conflicts_with_all(&["binaries", "all", "none"]),
        )
        .arg(
            Arg::new("binaries")
                .about("Determines what publishing options to use")
                .long("binaries")
                .required_unless_present_any(&["crates", "all", "none"])
                .conflicts_with_all(&["crates", "all", "none"]),
        );

    let matches = app.get_matches();

    let project_name = matches.value_of("name").unwrap(); // Required argument

    let mut project_type = ProjectType::Executable;
    if matches.is_present("lib") {
        project_type = ProjectType::Library;
    }

    let mut publish_option = PublishOptions::None;
    if matches.is_present("all") {
        publish_option = PublishOptions::All;
    } else if matches.is_present("crates") {
        publish_option = PublishOptions::Crates;
    } else if matches.is_present("binaries") {
        publish_option = PublishOptions::Binaries;
    }

    generate_project(project_name, project_type, publish_option)
        .expect("Failed to generate project");
}

fn generate_project(
    name: &str,
    project_type: ProjectType,
    publish_option: PublishOptions,
) -> std::io::Result<()> {
    let cargo_arg = match project_type {
        ProjectType::Executable => vec!["new", name],
        ProjectType::Library => vec!["new", name, "--lib"],
    };
    run_cargo_with_args(cargo_arg);

    fs::create_dir_all(format!("{}/.github/workflows", name))?;
    fs::write(format!("{}/.github/workflows/ci.yml", name), CI_WORKFLOW)?;
    let release_workflow_file = format!("{}/.github/workflows/release.yml", name);
    match publish_option {
        PublishOptions::All => fs::write(release_workflow_file, RELEASE_ALL_WORKFLOW)?,
        PublishOptions::Crates => fs::write(release_workflow_file, RELEASE_CRATES_WORKFLOW)?,
        PublishOptions::Binaries => fs::write(release_workflow_file, RELEASE_BINARIES_WORKFLOW)?,
        PublishOptions::None => (),
    };
    fs::write(
        format!("{}/README.md", name),
        format!(
            "![CI status](https://github.com/LimeEng/{}/workflows/CI/badge.svg)\n",
            name
        ),
    )
}

fn run_cargo_with_args(args: Vec<&str>) {
    Command::new("cargo")
        .args(args)
        .status()
        .expect("Failed to execute process");
}
