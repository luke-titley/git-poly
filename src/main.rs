//------------------------------------------------------------------------------
// Copyrite Luke Titley 2019
//------------------------------------------------------------------------------
mod branch_regex;
mod channel;
mod command;
mod filter;
mod git;
mod error;
mod io;
mod path;
mod repoiterator;
mod result;
mod status;
//------------------------------------------------------------------------------
use branch_regex::*;
use io::*;
use result::*;
//------------------------------------------------------------------------------
use regex;
use std;
use std::env;
use std::fs;
use std::process;

//------------------------------------------------------------------------------
fn mv(from: &str, to: &str) -> Result<()> {
    let mut from_path = path::PathBuf::new();
    let mut to_path = path::PathBuf::new();

    from_path.push(from);
    to_path.push(to);

    let (from_repo, from_rel) = git::relative_to_repo(&mut from_path)?;
    let (to_repo, to_rel) = git::relative_to_repo(&mut to_path)?;

    if from_path.exists() {
        // Remove the destionation if it exists
        if to_path.exists() {
            let output = process::Command::new("git")
                .args(&["rm", "-rf", to_rel.as_str()])
                .current_dir(to_repo.clone())
                .output()?;

            write_to_stderr(&to_repo, &output.stderr)?;
        }

        // Move the file
        fs::rename(&from_path, &to_path)?;

        // Remove the old file or folder
        {
            let output = process::Command::new("git")
                .args(&["rm", "-rf", from_rel.as_str()])
                .current_dir(from_repo.clone())
                .output()?;
            write_to_stderr(&to_repo, &output.stderr)?;
        }

        // Add the newfile or folder
        {
            let output = process::Command::new("git")
                .args(&["add", to_rel.as_str()])
                .current_dir(to_repo.clone())
                .output()?;
            write_to_stderr(&to_repo, &output.stderr)?;
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
struct Flags {
    path: regex::Regex,
    branch: BranchRegex,
}

//------------------------------------------------------------------------------
impl Flags {
    pub fn new() -> Result<Self> {
        let path = regex::Regex::new(r".*")?;
        Ok(Flags { path, branch: None })
    }
}

//------------------------------------------------------------------------------
fn main() -> Result<()> {
    // The flags
    let mut flags = Flags::new()?;

    // Grab the arguments
    let env_args: Vec<String> = env::args().collect();

    if env_args.len() == 1 {
        argument_error("Not enough arguments");
    }

    // Args is argv without the executable name
    let args = &env_args[1..];

    // Execute the sub commands
    let mut skip: usize = 0;
    for index in 0..args.len() {
        if skip == 0 {
            let arg = &args[index];
            match arg.as_str() {
                // Flags
                "--help" | "-h" => {
                    usage();
                    break;
                }
                "--path" | "-p" => {
                    if (index + 1) == args.len() {
                        argument_error(
                            "--path requires an expression \
                             (ie --path '.*')",
                        );
                    }
                    flags.path = regex::Regex::new(&(args[index + 1]))?;
                    skip = 1;
                }
                "--branch" | "-b" => {
                    if (index + 1) == args.len() {
                        argument_error(
                            "--branch requires an expression \
                             (ie --branch 'feature/foo.*')",
                        );
                    }
                    flags.branch = Some(regex::Regex::new(&(args[index + 1]))?);
                    skip = 1;
                }
                // Sub-commands
                "go" => {
                    if index + 1 == args.len() {
                        argument_error("go requires at least one git command");
                    }
                    command::go::run(&flags.path, &flags.branch, index + 1)?;
                    break;
                }
                "cmd" => {
                    if index + 1 == args.len() {
                        argument_error(
                            "cmd requires at least one shell command",
                        );
                    }
                    command::cmd::run(&flags.path, &flags.branch, index + 1)?;
                    break;
                }
                "add" => {
                    if index + 1 == args.len() {
                        let error = "Nothing specified, nothing added.
Maybe you wanted to say 'git add .'?";
                        argument_error(error);
                    }
                    command::add::run(&flags.path, index + 1)?;
                    break;
                }
                "grep" => {
                    if index + 1 == args.len() {
                        argument_error("Please provide the expression you would like to grep for");
                    }
                    command::grep::run(&flags.path, &flags.branch, args[index + 1].as_str())?;
                    break;
                }
                "ls-files" => {
                    command::ls_files::run(&flags.path, &flags.branch)?;
                    break;
                }
                "ls" => {
                    command::ls::run(&flags.path, &flags.branch)?;
                    break;
                }
                "clone" => {
                    command::clone::run(&flags.path)?;
                    break;
                }
                "commit" => {
                    if index + 2 >= args.len() {
                        argument_error(
                            "commit requires at least arguments -m and a message",
                        );
                    }

                    if args[index + 1] != "-m" {
                        argument_error("commit requires -m");
                    }

                    command::commit::run(
                        &flags.path,
                        &flags.branch,
                        args[index + 2].as_str(),
                    )?;
                    break;
                }
                "reset" => {
                    command::reset::run(&flags.path, &flags.branch)?;
                    break;
                }
                "status" => {
                    command::status::run(&flags.path, &flags.branch)?;
                    break;
                }
                "mv" => {
                    if index + 2 >= args.len() {
                        argument_error("mv requires a source and a dest");
                    }
                    mv(&args[index + 1], &args[index + 2])?;
                    break;
                }
                "replace" => {
                    if index + 2 >= args.len() {
                        argument_error(
                            "replace requires at least two arguments",
                        );
                    }
                    command::replace::run(&flags.path, &flags.branch, index + 1)?;
                    break;
                }
                _ => argument_error("argument not recognised"),
            }
        } else {
            skip -= 1;
        }
    }

    Ok(())
}
