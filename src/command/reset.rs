//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::filter;
use crate::io::{write_to_stderr, write_to_stdout};
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::Result;
//------------------------------------------------------------------------------
use std::process;

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
pub fn run(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
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
