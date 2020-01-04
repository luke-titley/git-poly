//------------------------------------------------------------------------------
use crate::branch_regex::BranchRegex;
use crate::filter;
use crate::io::{write_to_stderr};
use crate::path;
use crate::repoiterator::RepoIterator;
use crate::result::{handle_errors, Result};
//------------------------------------------------------------------------------
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process;
use std::thread;

//------------------------------------------------------------------------------
fn replace_in_file(
    from_regex: &regex::Regex,
    to_regex: &str,
    file_path: &path::Path,
) -> Result<()> {
    let mut output = Vec::<u8>::new();
    {
        let full_path = path::PathBuf::from(file_path);
        let input = fs::File::open(full_path.as_path())?;
        let buffered = BufReader::new(input);
        for line in buffered.lines() {
            let old_line = line?;
            let new_line =
                from_regex.replace_all(&old_line as &str, to_regex as &str);
            writeln!(output, "{0}", new_line)?;
        }
    }
    let mut input = fs::File::create(file_path)?;
    input.write_all(&output)?;

    Ok(())
}

//------------------------------------------------------------------------------
fn doit(
    branch_filter: &BranchRegex,
    path: &path::PathBuf,
    from: &str,
    to: &str,
) -> Result<()> {
    // Filter based on branch name
    if let Some(pattern) = branch_filter {
        if !filter::branch(&pattern, &path)? {
            return Ok(());
        }
    }

    let from_exp = regex::Regex::new(&from)?;

    let args = ["grep", "-l", from];
    let output = process::Command::new("git")
        .args(&args)
        .current_dir(path.clone())
        .output()?;

    // stderr
    write_to_stderr(&path, &output.stderr)?;

    // perform the find and replace
    if !output.stdout.is_empty() {
        let mut replace_threads = Vec::new();
        let stdout = BufReader::new(&output.stdout as &[u8]);
        for line in stdout.lines() {
            let file_path = path::Path::new(&path).join(line?);
            let from_regex = from_exp.clone();
            let to_regex = String::from(to);
            let replace_thread = thread::spawn(move || {
                handle_errors(replace_in_file(
                    &from_regex,
                    &to_regex,
                    &file_path,
                ))
            });

            replace_threads.push(replace_thread);
        }

        // Wait for all the replace threads to finish
        for replace_thread in replace_threads {
            replace_thread.join()?;
        }
    }

    Ok(())
}

//------------------------------------------------------------------------------
pub fn run(
    regex: &regex::Regex,
    branch_regex: &BranchRegex,
    args_pos: usize,
) -> Result<()> {
    let mut threads = Vec::new();

    let args: Vec<String> = env::args().collect();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        // Get hold of the from and to
        let from = args[args_pos + 1].clone();
        let to = args[args_pos + 2].clone();
        let branch_filter = branch_regex.clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {
            handle_errors(doit(&branch_filter, &path, &from, &to))
        });
        threads.push(thread);
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join()?;
    }

    Ok(())
}
