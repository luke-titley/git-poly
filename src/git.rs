use super::error;
use super::io;
use super::path;
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

//------------------------------------------------------------------------------
pub fn relative_to_repo(
    path: &mut path::PathBuf,
) -> result::Result<(path::PathBuf, String)> {
    for parent in path.ancestors() {
        if !parent.as_os_str().is_empty() {
            let mut repo = path::PathBuf::new(); // TODO LT: Wanted to keep this around but need nightly to use 'clear'.
            repo.push(parent);
            repo.pop();
            repo.push(".git");

            if repo.exists() {
                repo.pop();
                let relative_path = result::get(
                    path.as_path().strip_prefix(repo.as_path())?.to_str(),
                )?;
                return Ok((repo, relative_path.to_string()));
            }
        }
    }

    Err(error::Error::RelativeToRepo())
}
