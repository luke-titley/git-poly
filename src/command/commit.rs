//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::filter;
use crate::io::{write_to_stderr, write_to_stdout};
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::{handle_errors, Result};
//------------------------------------------------------------------------------
use std::io::{BufRead, BufReader};
use std::process;
use std::str::FromStr;
use std::thread;

//------------------------------------------------------------------------------
fn doit(
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
pub fn run(
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
            handle_errors(doit(&message, &c, &branch_filter, &path))
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}
