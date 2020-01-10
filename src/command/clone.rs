//------------------------------------------------------------------------------
use crate::io::{write_to_stderr, write_to_stdout};
use crate::path;
use crate::result::{handle_errors, Result};
//------------------------------------------------------------------------------
use std::fs;
use std::io::BufRead;
use std::process;
use std::thread;

//------------------------------------------------------------------------------
fn doit(dirs: &regex::Regex, url: &str) -> Result<()> {
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
pub fn run(regex: &regex::Regex) -> Result<()> {
    let mut threads = Vec::new();

    // This will break the git repo url https/http or git into three parts
    // The protocol, the path and the option .git extension
    const GIT_REPO_URL: &str = r"^([a-zA-Z0-9-]+@[a-zA-Z0-9.-]+:|https?://[a-zA-Z0-9.-]+/)([a-zA-Z/-]+)(\.git)?";

    let dirs_regex = regex::Regex::new(GIT_REPO_URL);

    // Loop over the lines in stdin
    let stdin = std::io::stdin();
    for l in stdin.lock().lines() {
        let line = l?;
        if regex.is_match(line.as_str()) {
            let dirs = dirs_regex.clone()?;
            threads.push(thread::spawn(move || {
                handle_errors(doit(&dirs, line.as_str()))
            }));
        }
    }

    for thread in threads {
        thread.join()?;
    }

    Ok(())
}
