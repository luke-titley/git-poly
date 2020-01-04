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
use regex;
use std;

//------------------------------------------------------------------------------
struct Flags {
    path: regex::Regex,
    branch: branch_regex::BranchRegex,
}

//------------------------------------------------------------------------------
impl Flags {
    pub fn new() -> result::Result<Self> {
        let path = regex::Regex::new(r".*")?;
        Ok(Flags { path, branch: None })
    }
}

//------------------------------------------------------------------------------
fn main() -> result::Result<()> {
    // The flags
    let mut flags = Flags::new()?;

    // Grab the arguments
    let env_args: Vec<String> = std::env::args().collect();

    if env_args.len() == 1 {
        io::argument_error("Not enough arguments");
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
                    io::usage();
                    break;
                }
                "--path" | "-p" => {
                    if (index + 1) == args.len() {
                        io::argument_error(
                            "--path requires an expression \
                             (ie --path '.*')",
                        );
                    }
                    flags.path = regex::Regex::new(&(args[index + 1]))?;
                    skip = 1;
                }
                "--branch" | "-b" => {
                    if (index + 1) == args.len() {
                        io::argument_error(
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
                        io::argument_error("go requires at least one git command");
                    }
                    command::go::run(&flags.path, &flags.branch, index + 1)?;
                    break;
                }
                "cmd" => {
                    if index + 1 == args.len() {
                        io::argument_error(
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
                        io::argument_error(error);
                    }
                    command::add::run(&flags.path, index + 1)?;
                    break;
                }
                "grep" => {
                    if index + 1 == args.len() {
                        io::argument_error("Please provide the expression you would like to grep for");
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
                        io::argument_error(
                            "commit requires at least arguments -m and a message",
                        );
                    }

                    if args[index + 1] != "-m" {
                        io::argument_error("commit requires -m");
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
                        io::argument_error("mv requires a source and a dest");
                    }
                    command::mv::run(&args[index + 1], &args[index + 2])?;
                    break;
                }
                "replace" => {
                    if index + 2 >= args.len() {
                        io::argument_error(
                            "replace requires at least two arguments",
                        );
                    }
                    command::replace::run(&flags.path, &flags.branch, index + 1)?;
                    break;
                }
                _ => io::argument_error("argument not recognised"),
            }
        } else {
            skip -= 1;
        }
    }

    Ok(())
}
