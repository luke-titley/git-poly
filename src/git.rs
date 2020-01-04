use super::path;
use super::error;
use super::io;
use super::result;
//------------------------------------------------------------------------------
use std::io::BufRead;
use std::io::BufReader;
use std::process;

//------------------------------------------------------------------------------
pub fn get_branch_name(path: &path::PathBuf) -> result::Result<String> {
    let output = process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path.clone())
        .output()?;

    io::write_to_stderr(&path, &output.stderr)?;

    let stdout = BufReader::new(&output.stdout as &[u8]);
    let result: Vec<_> = stdout.lines().collect();

    if result.is_empty() {
        return Ok("HEADLESS".to_string());
    }

    match result[0].as_ref() {
        Ok(r) => Ok(r.to_string()),
        Err(_) => Err(error::Error::None()),
    }
}

