use super::channel;
use super::path;
use super::result;
//------------------------------------------------------------------------------
use regex;
use std::fs;
use std::io::Write;
use std::thread;
use std::vec;

//------------------------------------------------------------------------------
fn list_repos(
    regex: &regex::Regex,
    send: &channel::PathSender,
) -> result::Result<()> {
    let mut current_dir = path::PathBuf::new();
    current_dir.push(".");

    type Paths = vec::Vec<path::PathBuf>;
    let mut paths = Paths::new();

    paths.push(current_dir);

    // Walk over the directory
    while let Some(path) = paths.pop() {
        match fs::read_dir(path.clone()) {
            Ok(dir) => {
                for entry in dir {
                    let p = entry?.path();
                    if p.is_dir() {
                        let mut p_buf = p.to_path_buf();
                        if let Some(name) = p.file_name() {
                            match name.to_str() {
                                Some(".git") => {
                                    // We've found a git repo, send it back
                                    p_buf.pop();
                                    let repo_path = p_buf.as_path();
                                    if regex.is_match(result::get(
                                        repo_path.to_str(),
                                    )?) {
                                        send.send(Some(p_buf))?;
                                    }
                                }
                                _ => {
                                    paths.push(p_buf);
                                }
                            }
                        }
                    }
                }
            }
            Err(error) => {
                let mut stderr = std::io::stderr();
                writeln!(stderr, "{0} '{1}'", error, path.display())?;
            }
        }
    }

    // Send an empty message to say we're done
    send.send(None)?;

    Ok(())
}

//------------------------------------------------------------------------------
// RepoIterator
//------------------------------------------------------------------------------
pub struct RepoIterator {
    recv: channel::PathReceiver,
}

//------------------------------------------------------------------------------
impl RepoIterator {
    pub fn new(regex: &regex::Regex) -> Self {
        let (send, recv) = channel::path_channel();

        // Kick off the traversal thread. It's detached by default.
        let regex_copy = regex.clone();
        thread::spawn(move || {
            result::handle_errors(list_repos(&regex_copy, &send));
        });

        // Make the new thread object
        RepoIterator { recv }
    }
}

//------------------------------------------------------------------------------
impl Iterator for RepoIterator {
    type Item = path::PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        match self.recv.recv() {
            Ok(result) => result,
            Err(error) => match writeln!(std::io::stderr(), "{0}", error) {
                _ => None,
            },
        }
    }
}
