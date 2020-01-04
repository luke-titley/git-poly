//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::channel;
use crate::error;
use crate::filter;
use crate::git;
use crate::io::write_to_stderr;
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::{get, handle_errors, Result};
use crate::status::*;
//------------------------------------------------------------------------------
use colored::*;
//------------------------------------------------------------------------------
use std::io::{BufRead, BufReader};
use std::iter::FromIterator;
use std::process;
use std::thread;
use std::vec;

//------------------------------------------------------------------------------
fn convert_to_status(input: &str) -> Result<Status> {
    match input {
        "??" => Ok((Tracking::Untracked, Staging::Untracked)),
        " M" => Ok((Tracking::Unstaged, Staging::Modified)),
        " A" => Ok((Tracking::Unstaged, Staging::Added)),
        " D" => Ok((Tracking::Unstaged, Staging::Deleted)),
        "M " => Ok((Tracking::Staged, Staging::Modified)),
        "A " => Ok((Tracking::Staged, Staging::Added)),
        "D " => Ok((Tracking::Staged, Staging::Deleted)),
        "UU" => Ok((Tracking::Unmerged, Staging::BothModified)),
        _ => Err(error::Error::UnableToParseStatus),
    }
}

//------------------------------------------------------------------------------
fn match_color(tracking: &Tracking) -> &'static str {
    match *tracking {
        Tracking::Unstaged => "red",
        Tracking::Untracked => "red",
        Tracking::Unmerged => "red",
        _ => "green",
    }
}

//------------------------------------------------------------------------------
fn print_title(tracking: &Tracking) {
    match *tracking {
        Tracking::Unstaged => {
            println!("Changes not staged for commit:");
            println!("  (use \"git add <file>...\" to include in what will be committed)");
            println!();
        }
        Tracking::Untracked => {
            println!("Untracked files:");
            println!("  (use \"git add <file>...\" to include in what will be committed)");
            println!();
        }
        Tracking::Unmerged => {
            println!("You have unmerged paths.");
            println!("  (fix conflicts and run \"git commit\")");
            println!("  (use \"git merge --abort\" to abort the merge)");
            println!();
            println!("Unmerged paths:");
            println!("  (use \"git add <file>...\" to mark resolution)");
        }
        _ => {
            println!("Changes to be commited:");
            println!();
        }
    }
}

//------------------------------------------------------------------------------
fn status_thread(
    sender: &channel::StatusSender,
    path: &path::PathBuf,
    splitter: &regex::Regex,
    branch_filter: &BranchRegex,
) -> Result<()> {
    // Filter based on branch name
    if let Some(pattern) = branch_filter {
        if !filter::branch(&pattern, path)? {
            return Ok(());
        }
    }

    let branch_name = git::get_branch_name(path)?;

    let args = ["status", "--porcelain"];
    let output = process::Command::new("git")
        .args(&args)
        .current_dir(path.clone())
        .output()?;

    write_to_stderr(path, &output.stderr)?;

    let stdout = BufReader::new(&output.stdout as &[u8]);

    for line_result in stdout.lines() {
        let line = line_result?;
        let mut file_path = path::PathBuf::new();
        let split: Vec<_> = splitter.captures_iter(line.as_str()).collect();

        if !split.is_empty() {
            let status = convert_to_status(&split[0][1])?;
            let file = &split[0][2];
            file_path.push(path.clone());
            file_path.push(file);
            sender.send((
                branch_name.clone(),
                status,
                get(file_path.to_str())?.to_string(),
            ))?;
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
struct StatusIteration<'a> {
    msg: &'a StatusMsg,
    print_branch: bool,
    print_tracking: bool,
    color: &'static str,
}

//------------------------------------------------------------------------------
// StatusIterator
//------------------------------------------------------------------------------
struct StatusIterator<'a> {
    statuses: &'a vec::Vec<StatusMsg>,
    index: usize,
}

//------------------------------------------------------------------------------
impl<'a> StatusIterator<'a> {
    pub fn new(statuses: &'a vec::Vec<StatusMsg>) -> Self {
        StatusIterator { statuses, index: 0 }
    }
}

//------------------------------------------------------------------------------
impl<'a> std::iter::Iterator for StatusIterator<'a> {
    type Item = StatusIteration<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        // We're out of range
        if self.statuses.len() <= self.index {
            None
        } else {
            // The common values
            let index = self.index;
            let status = &(self.statuses[index]);
            let color = match_color(&(status.1).0);

            // Increment
            self.index += 1;

            // We're on the first entry
            if index == 0 {
                Some(StatusIteration {
                    msg: status,
                    print_branch: true,
                    print_tracking: true,
                    color,
                })

            // We're on the next entry
            } else {
                let previous_status = &(self.statuses[index - 1]);
                let print_branch = status.0 != previous_status.0;
                let print_tracking =
                    print_branch || (status.1).0 != (previous_status.1).0;

                Some(StatusIteration {
                    msg: status,
                    print_branch,
                    print_tracking,
                    color,
                })
            }
        }
    }
}

//------------------------------------------------------------------------------
pub fn run(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
    let (send, recv) = channel::status_channel();

    let splitter_def = regex::Regex::new(r"(UU| M|M |MM|A | D|D |\?\?) (.*)")?;

    let mut threads = Vec::new();
    for path in RepoIterator::new(regex) {
        let sender = send.clone();
        let splitter = splitter_def.clone();
        let branch_filter = branch_regex.clone();

        let thread = thread::spawn(move || {
            handle_errors(status_thread(
                &sender,
                &path,
                &splitter,
                &branch_filter,
            ))
        });

        threads.push(thread);
    }
    drop(send);

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    // Store all the changes in a vector;
    let mut changes = Vec::from_iter(recv.iter());
    changes.sort();

    // Iterate over the changes printing in the git way
    for i in StatusIterator::new(&changes) {
        let (branch, (tracking, staging), path) = i.msg;

        // Branch name if necessary
        if i.print_branch {
            println!();
            println!("on branch {0}", branch.cyan());
        }

        // Tracking title if necessary
        if i.print_tracking {
            if !i.print_branch {
                println!();
            }
            print_title(&tracking);
        }

        // Staging info
        match staging {
            Staging::Modified => {
                print!("{0}", "        modified:   ".color(i.color))
            }
            Staging::Deleted => {
                print!("{0}", "        deleted:   ".color(i.color))
            }
            Staging::Added => {
                print!("{0}", "        new file:   ".color(i.color))
            }
            Staging::BothModified => {
                print!("{0}", "        both modified:   ".color(i.color))
            }
            _ => print!("        "),
        }
        println!("{0}", path.color(i.color));
    }

    Ok(())
}
