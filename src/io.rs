use super::path;
use super::result;
//------------------------------------------------------------------------------
use colored::*;
use std;

//------------------------------------------------------------------------------
// Usage
//------------------------------------------------------------------------------
pub const USAGE: &str = "
USAGE:
    git poly [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -p, --path <regex>        Filter by repo file path using given expression
    -b, --branch <regex>      Filter by current branch using given expression

SUBCOMMANDS
    go <git command>          Execute a git command in each repo
    cmd <comand>              Execute a shell command in each repo
    ls                        List all the git repos discovered

    clone                     Clone the repositories listed in stdin
    add [-u] [<pathspec>...]  Add file contents to the index of it's repo
    commit [-m] <message>     Record changes to the repository
    grep <pattern>            Print lines matching a pattern
    ls-files                  Show information about files in the index and the working tree
    mv <from> <to>            Move or rename a file, a directory, or a symlink
    reset                     Reset current HEAD to the specified state
    status                    Show the merged working tree status of all the repos

    replace <from> <to>       Find and replace all occurances of FROM with TO
";

//------------------------------------------------------------------------------
pub fn argument_error(msg: &str) {
    println!("error: {0}\n{1}", msg, USAGE);
    std::process::exit(1);
}

//------------------------------------------------------------------------------
pub fn usage() {
    println!("{0}", USAGE);
}

//------------------------------------------------------------------------------
pub fn write_to_out(
    handle: &mut dyn std::io::Write,
    repo: &path::PathBuf,
    output: &[u8],
) -> result::Result<()> {
    let display = result::get(repo.as_path().to_str())?;

    writeln!(handle, "{0}", display.cyan())?;
    handle.write_all(&output)?;
    writeln!(handle)?;

    Ok(())
}

//------------------------------------------------------------------------------
pub fn write_to_stdout(
    repo: &path::PathBuf,
    output: &[u8],
) -> result::Result<()> {
    // stdout
    if !output.is_empty() {
        let stdout = std::io::stdout();
        {
            let mut handle = stdout.lock();
            write_to_out(&mut handle, repo, output)?;
        }
    }
    Ok(())
}

//------------------------------------------------------------------------------
pub fn write_to_stderr(
    repo: &path::PathBuf,
    output: &[u8],
) -> result::Result<()> {
    // stderr
    if !output.is_empty() {
        let stderr = std::io::stderr();
        {
            let mut handle = stderr.lock();
            write_to_out(&mut handle, repo, output)?;
        }
    }
    Ok(())
}
