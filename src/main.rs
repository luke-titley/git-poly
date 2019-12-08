use std::env;
use std::fs;
use std::io;
use std::path;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::vec;

type Paths = vec::Vec<path::PathBuf>;
type Error = io::Result<()>;
type Msg = Option<path::PathBuf>;
type Sender = mpsc::Sender<Msg>;
type Receiver = mpsc::Receiver<Msg>;

//------------------------------------------------------------------------------
// list_repos
//------------------------------------------------------------------------------
fn list_repos(send: &Sender) -> Error {
    let current_dir = env::current_dir()?;

    let mut paths = Paths::new();

    paths.push(current_dir);

    // Walk over the directory
    while !paths.is_empty() {
        let path = paths.pop().unwrap();
        for entry in fs::read_dir(path)? {
            let p = entry?.path();
            if p.is_dir() {
                let mut p_buf = p.to_path_buf();
                let name = p.file_name().unwrap().to_str();
                match name {
                    Some(".git") => {
                        // We've found a git repo, send it back
                        p_buf.pop();
                        send.send(Some(p_buf)).unwrap();
                    }
                    _ => {
                        paths.push(p_buf);
                    }
                }
            }
        }
    }

    // Send an empty message to say we're done
    send.send(None).unwrap();

    Ok(())
}

//------------------------------------------------------------------------------
// RepoIterator
//------------------------------------------------------------------------------
struct RepoIterator {
    recv: Receiver,
}

//------------------------------------------------------------------------------
impl RepoIterator {
    fn new() -> Self {
        let (send, recv): (Sender, Receiver) = mpsc::channel();

        // Kick off the traversal thread. It's detached by default.
        thread::spawn(move || list_repos(&send).unwrap());

        // Make the new thread object
        RepoIterator { recv }
    }
}

//------------------------------------------------------------------------------
impl Iterator for RepoIterator {
    type Item = path::PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.recv.recv().unwrap()
    }
}

//------------------------------------------------------------------------------
fn write_to_out(
    handle: &mut dyn io::Write,
    repo: &path::PathBuf,
    output: &[u8],
) -> io::Result<()> {
    let display = repo.as_path().to_str().unwrap();

    writeln!(handle)?;
    writeln!(handle, "# {0}", display)?;
    writeln!(handle, "# {0}", "-".repeat(display.len()))?;
    handle.write_all(&output)?;
    writeln!(handle)?;

    Ok(())
}

//------------------------------------------------------------------------------
fn write_to_stdout(repo: &path::PathBuf, output: &[u8]) {
    // stdout
    if !output.is_empty() {
        let stdout = io::stdout();
        {
            let mut handle = stdout.lock();
            write_to_out(&mut handle, repo, output).unwrap();
        }
    }
}

//------------------------------------------------------------------------------
fn write_to_stderr(repo: &path::PathBuf, output: &[u8]) {
    // stdout
    if !output.is_empty() {
        let stderr = io::stderr();
        {
            let mut handle = stderr.lock();
            write_to_out(&mut handle, repo, output).unwrap();
        }
    }
}

//------------------------------------------------------------------------------
fn main() -> Error {
    let mut args = env::args().enumerate();
    args.next();
    for arg in args {
        match arg {
            (index, arguement) => {
                match arguement.as_str() {
                    "go" => {
                        let mut threads = Vec::new();

                        // Loop through the results of what the walker is outputting
                        for path in RepoIterator::new() {
                            // Execute a new thread for processing this result
                            threads.push(thread::spawn(move || {
                                let args: Vec<String> = env::args().collect();
                                let output = process::Command::new("git")
                                    .args(&args[index + 1..])
                                    .current_dir(path.clone())
                                    .output()
                                    .unwrap();

                                // stdout/stderr
                                write_to_stdout(&path, &output.stdout);
                                write_to_stderr(&path, &output.stderr);
                            }));
                        }

                        // Wait for all the threads to finish
                        for thread in threads {
                            thread.join().unwrap();
                        }

                        // We're done now
                        break;
                    }
                    "ls" => {
                        let (send, recv): (Sender, Receiver) = mpsc::channel();
                        let mut threads = Vec::new();

                        threads.push(thread::spawn(move || {
                            list_repos(&send).unwrap()
                        }));

                        // Loop through the results of what the walker is outputting
                        while let Some(path) = recv.recv().unwrap() {
                            let display = path.as_path().to_str().unwrap();
                            println!("# {0}", display);
                        }

                        for thread in threads {
                            thread.join().unwrap();
                        }
                    }
                    "replace" => {
                        panic!("Not implemented yet");
                    }
                    _ => panic!("Incorrect arguments"),
                }
            }
        }
    }

    Ok(())
}
