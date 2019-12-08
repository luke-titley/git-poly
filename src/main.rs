use std::env;
use std::fs;
use std::io;
use std::io::Write;
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
fn main() -> Error {
    let mut args = env::args().enumerate();
    args.next();
    for arg in args {
        match arg {
            (index, arguement) => {
                match arguement.as_str() {
                    "go" => {
                        let (send, recv): (Sender, Receiver) = mpsc::channel();
                        let mut threads = Vec::new();

                        threads.push(thread::spawn(move || {
                            list_repos(&send).unwrap()
                        }));

                        // Loop through the results of what the walker is outputting
                        while let Some(path) = recv.recv().unwrap() {
                            // Execute a new thread for processing this result
                            threads.push(thread::spawn(move || {
                                let args: Vec<String> = env::args().collect();
                                let output = process::Command::new("git")
                                    .args(&args[index + 1..])
                                    .current_dir(path.clone())
                                    .output()
                                    .unwrap();

                                // stdout
                                if !output.stdout.is_empty() {
                                    let stdout = io::stdout();
                                    {
                                        let _ = stdout.lock();
                                        let display =
                                            path.as_path().to_str().unwrap();
                                        println!("");
                                        println!("# {0}", display);
                                        println!(
                                            "# {0}",
                                            "-".repeat(display.len())
                                        );
                                        io::stdout()
                                            .write_all(&output.stdout)
                                            .unwrap();
                                        println!("");
                                    }
                                }

                                // stderr
                                io::stderr().write_all(&output.stderr).unwrap();
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
                    },
                    "replace" => {
                        panic!("Not implemented yet");
                    },
                    _ => panic!("Incorrect arguments"),
                }
            }
        }
    }

    Ok(())
}
