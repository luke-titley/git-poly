//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::filter;
use crate::io::{write_to_stderr, write_to_stdout};
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::{handle_errors, Result};
//------------------------------------------------------------------------------
use std::env;
use std::process;
use std::thread;

//------------------------------------------------------------------------------
fn doit(
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
    let output = process::Command::new("git")
        .args(&args[args_pos + 1..])
        .current_dir(path.clone())
        .output()?;

    // stdout/stderr
    write_to_stdout(&path, &output.stdout)?;
    write_to_stderr(&path, &output.stderr)?;

    Ok(())
}

//------------------------------------------------------------------------------
pub fn run(
    path_regex: &regex::Regex,
    branch_regex: &BranchRegex,
    args_pos: usize,
) -> Result<()> {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(path_regex) {
        let branch_filter = branch_regex.clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {
            handle_errors(doit(&path, &branch_filter, args_pos))
        });

        threads.push(thread);
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}
