//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::filter;
use crate::io;
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::{handle_errors, Result};
//------------------------------------------------------------------------------
use std::io::{BufRead, BufReader};
use std::process;
use std::thread;

//------------------------------------------------------------------------------
fn doit(
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
        .args(&["ls-files"])
        .current_dir(path.clone())
        .output()?;

    io::write_to_stderr(&path, &output.stderr)?;

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
pub fn run(regex: &regex::Regex, branch_regex: &BranchRegex) -> Result<()> {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        let branch_filter = branch_regex.clone();

        threads.push(thread::spawn(move || {
            handle_errors(doit(&branch_filter, &path))
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}
