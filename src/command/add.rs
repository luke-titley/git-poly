//------------------------------------------------------------------------------
use crate::git;
use crate::io::{write_to_stderr, write_to_stdout};
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::{handle_errors, Result};
//------------------------------------------------------------------------------
use std::env;
use std::process;
use std::thread;

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
fn add_entry(path: &mut path::PathBuf) -> Result<()> {
    let (repo, relative_path) = git::relative_to_repo(path)?;
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
pub fn run(regex: &regex::Regex, args_pos: usize) -> Result<()> {
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
