//------------------------------------------------------------------------------
use crate::git;
use crate::io::write_to_stderr;
use crate::path;
use crate::result::Result;
//------------------------------------------------------------------------------
use std::fs;
use std::process;

//------------------------------------------------------------------------------
pub fn run(from: &str, to: &str) -> Result<()> {
    let mut from_path = path::PathBuf::new();
    let mut to_path = path::PathBuf::new();

    from_path.push(from);
    to_path.push(to);

    let (from_repo, from_rel) = git::relative_to_repo(&mut from_path)?;
    let (to_repo, to_rel) = git::relative_to_repo(&mut to_path)?;

    if from_path.exists() {
        // Remove the destionation if it exists
        if to_path.exists() {
            let output = process::Command::new("git")
                .args(&["rm", "-rf", to_rel.as_str()])
                .current_dir(to_repo)
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
