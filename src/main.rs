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
use channel::*;
use error::*;
use io::*;
use repoiterator::*;
use result::*;
use status::*;
//------------------------------------------------------------------------------
use regex;
use std;
use std::env;
use std::fs;
use std::iter::FromIterator;
use std::process;
use std::str::FromStr;
use std::thread;
use std::vec;

use std::io::BufRead;
use std::io::BufReader;

use colored::*;

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
        _ => Err(Error::UnableToParseStatus),
    }
}

//------------------------------------------------------------------------------
fn cmd_thread(
    path: &path::PathBuf,
    branch_filter: &BranchRegex,
    args_pos: usize,
) -> Result<()> {
    // Filter based on branch name
    if let Some(pattern) = branch_filter {
        if !filter::branch(&pattern, &path)? {
            return Ok(());
        }
    }

    let args: Vec<String> = env::args().collect();
    let args_ref = &args[args_pos + 1..];
    let output = process::Command::new(args_ref[0].clone())
        .args(&args_ref[1..])
        .current_dir(path.clone())
        .output()?;

    // stdout/stderr
    write_to_stdout(&path, &output.stdout)?;
    write_to_stderr(&path, &output.stderr)?;

    Ok(())
}

//------------------------------------------------------------------------------
fn cmd(
    regex: &regex::Regex,
    branch_regex: &BranchRegex,
    args_pos: usize,
) -> Result<()> {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        let branch_filter = branch_regex.clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {
            handle_errors(cmd_thread(&path, &branch_filter, args_pos))
        });
        threads.push(thread);
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn add_changed_thread(path: &path::PathBuf) -> Result<()> {
    let output = process::Command::new("git")
        .args(&["add", "-u"])
        .current_dir(path.clone())
        .output()?;

    // stdout/stderr
    write_to_stdout(&path, &output.stdout)?;
    write_to_stderr(&path, &output.stderr)?;

    Ok(())
}

//------------------------------------------------------------------------------
fn add_changed(regex: &regex::Regex) -> Result<()> {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        // Execute a new thread for processing this result
        let thread =
            thread::spawn(move || handle_errors(add_changed_thread(&path)));

        threads.push(thread);
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn relative_to_repo(
    path: &mut path::PathBuf,
) -> Result<(path::PathBuf, String)> {
    for parent in path.ancestors() {
        if !parent.as_os_str().is_empty() {
            let mut repo = path::PathBuf::new(); // TODO LT: Wanted to keep this around but need nightly to use 'clear'.
            repo.push(parent);
            repo.pop();
            repo.push(".git");

            if repo.exists() {
                repo.pop();
                let relative_path =
                    get(path.as_path().strip_prefix(repo.as_path())?.to_str())?;
                return Ok((repo, relative_path.to_string()));
            }
        }
    }

    Err(Error::RelativeToRepo())
}

//------------------------------------------------------------------------------
fn add_entry(path: &mut path::PathBuf) -> Result<()> {
    let (repo, relative_path) = relative_to_repo(path)?;
    let args = ["add", relative_path.as_str()];
    let output = process::Command::new("git")
        .args(&args)
        .current_dir(repo.clone())
        .output()?;

    // stdout/stderr
    write_to_stdout(&repo, &output.stdout)?;
    write_to_stderr(&repo, &output.stderr)?;

    Ok(())
}

//------------------------------------------------------------------------------
fn grep_thread(
    expr: &str,
    branch_filter: &BranchRegex,
    path: &path::PathBuf,
) -> Result<()> {
    // Filter based on branch name
    if let Some(pattern) = branch_filter {
        if !filter::branch(&pattern, &path)? {
            return Ok(());
        }
    }

    let output = process::Command::new("git")
        .args(&["grep", expr])
        .current_dir(path.clone())
        .output()?;

    write_to_stderr(&path, &output.stderr)?;

    let outstream = std::io::stdout();
    {
        let _handle = outstream.lock();
        let stdout = BufReader::new(&output.stdout as &[u8]);
        let flat_path = path.as_path().join(path::Path::new(""));
        for line in stdout.lines() {
            print!("{0}", flat_path.display());
            println!("{0}", line?);
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn grep(
    regex: &regex::Regex,
    branch_regex: &BranchRegex,
    expression: &str,
) -> Result<()> {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        let expr = expression.to_string();
        let branch_filter = branch_regex.clone();

        threads.push(thread::spawn(move || {
            handle_errors(grep_thread(&expr, &branch_filter, &path))
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn add(regex: &regex::Regex, args_pos: usize) -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut minus_u = false;
    for item in args.iter().skip(args_pos + 1) {
        match item.as_str() {
            "-u" => {
                if !minus_u {
                    minus_u = true;
                    add_changed(regex)?;
                }
            }
            file_path => {
                let mut path = path::PathBuf::from(file_path);
                add_entry(&mut path)?;
            }
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn reset_all(path: path::PathBuf) -> Result<()> {
    let output = process::Command::new("git")
        .args(&["reset"])
        .current_dir(path.clone())
        .output()?;

    // stdout/stderr
    write_to_stdout(&path, &output.stdout)?;
    write_to_stderr(&path, &output.stderr)?;

    Ok(())
}

//------------------------------------------------------------------------------
fn reset(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
    // Filtered traversal
    if let Some(pattern) = branch_regex {
        for path in RepoIterator::new(regex) {
            if filter::branch(&pattern, &path)? {
                reset_all(path)?;
            }
        }

    // Unfiltered traversal
    } else {
        for path in RepoIterator::new(regex) {
            reset_all(path)?;
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn ls(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
    // Filtered traversal
    if let Some(pattern) = branch_regex {
        for path in RepoIterator::new(regex) {
            if filter::branch(&pattern, &path)? {
                let display = get(path.as_path().to_str())?;
                println!("{0}", display);
            }
        }

    // Unfiltered traversal
    } else {
        for path in RepoIterator::new(regex) {
            let display = get(path.as_path().to_str())?;
            println!("{0}", display);
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn clone_thread(dirs: &regex::Regex, url: &str) -> Result<()> {
    let result: Vec<_> = dirs.captures_iter(url).collect();

    println!("Matching {0}", url);

    const FOLDER: usize = 2;

    if !result.is_empty() {
        for i in result {
            // Clone all the matches
            let mut path = path::PathBuf::from(".");
            path.push(&i[FOLDER]);

            //path = fs::canonicalize(path)?;

            // Make the folder
            fs::create_dir_all(path.as_path())?;

            // Clone the repo
            let output = process::Command::new("git")
                .args(&["clone", url, "."])
                .current_dir(path.as_path())
                .output()?;

            // stdout/stderr
            write_to_stdout(&path, &output.stdout)?;
            write_to_stderr(&path, &output.stderr)?;
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn clone(regex: &regex::Regex) -> Result<()> {
    let mut threads = Vec::new();

    // This will break the git repo url https/http or git into three parts
    // The protocol, the path and the option .git extension
    const GIT_REPO_URL : &str = r"^([a-zA-Z0-9-]+@[a-zA-Z0-9.-]+:|https?://[a-zA-Z0-9.-]+/)([a-zA-Z/-]+)(\.git)?";

    let dirs_regex = regex::Regex::new(GIT_REPO_URL);

    // Loop over the lines in stdin
    let stdin = std::io::stdin();
    for l in stdin.lock().lines() {
        let line = l?;
        if regex.is_match(line.as_str()) {
            let dirs = dirs_regex.clone()?;
            threads.push(thread::spawn(move || {
                handle_errors(clone_thread(&dirs, line.as_str()))
            }));
        }
    }

    for thread in threads {
        thread.join()?;
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn command_thread(
    message: &str,
    c: &regex::Regex,
    branch_filter: &BranchRegex,
    path: &path::PathBuf,
) -> Result<()> {
    // Filter based on branch name
    if let Some(pattern) = branch_filter {
        if !filter::branch(&pattern, &path)? {
            return Ok(());
        }
    }

    let args = ["status", "--porcelain"];
    let output = process::Command::new("git")
        .args(&args)
        .current_dir(path.clone())
        .output()?;

    write_to_stderr(&path, &output.stderr)?;

    // Search for modifications
    let stdout = BufReader::new(&output.stdout as &[u8]);
    let mut lines = stdout.lines();
    let has_modifications = {
        loop {
            if let Some(result) = lines.next() {
                let line = result?;
                if c.is_match(line.as_str()) {
                    break true;
                }
            } else {
                break false;
            }
        }
    };

    // If we have modifications then do a commit
    if has_modifications {
        let output = process::Command::new("git")
            .args(&["commit", "-m", message])
            .current_dir(path.clone())
            .output()?;

        write_to_stderr(&path, &output.stderr)?;
        write_to_stdout(&path, &output.stdout)?;
    }

    Ok(())
}

//------------------------------------------------------------------------------
fn commit(
    regex: &regex::Regex,
    branch_regex: &BranchRegex,
    msg: &str,
) -> Result<()> {
    let mut threads = Vec::new();

    let changes = regex::Regex::new(r"^(M|A|D) .*")?;

    for path in RepoIterator::new(regex) {
        let message = String::from_str(msg)?;
        let c = changes.clone();
        let branch_filter = branch_regex.clone();

        threads.push(thread::spawn(move || {
            handle_errors(command_thread(&message, &c, &branch_filter, &path))
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
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
    sender: &StatusSender,
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
fn status(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
    let (send, recv) = status_channel();

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

//------------------------------------------------------------------------------
fn mv(from: &str, to: &str) -> Result<()> {
    let mut from_path = path::PathBuf::new();
    let mut to_path = path::PathBuf::new();

    from_path.push(from);
    to_path.push(to);

    let (from_repo, from_rel) = relative_to_repo(&mut from_path)?;
    let (to_repo, to_rel) = relative_to_repo(&mut to_path)?;

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
                    cmd(&flags.path, &flags.branch, index + 1)?;
                    break;
                }
                "add" => {
                    if index + 1 == args.len() {
                        let error = "Nothing specified, nothing added.
Maybe you wanted to say 'git add .'?";
                        argument_error(error);
                    }
                    add(&flags.path, index + 1)?;
                    break;
                }
                "grep" => {
                    if index + 1 == args.len() {
                        argument_error("Please provide the expression you would like to grep for");
                    }
                    grep(&flags.path, &flags.branch, args[index + 1].as_str())?;
                    break;
                }
                "ls-files" => {
                    command::ls_files::run(&flags.path, &flags.branch)?;
                    break;
                }
                "ls" => {
                    ls(&flags.path, &flags.branch)?;
                    break;
                }
                "clone" => {
                    clone(&flags.path)?;
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

                    commit(
                        &flags.path,
                        &flags.branch,
                        args[index + 2].as_str(),
                    )?;
                    break;
                }
                "reset" => {
                    reset(&flags.path, &flags.branch)?;
                    break;
                }
                "status" => {
                    status(&flags.path, &flags.branch)?;
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
