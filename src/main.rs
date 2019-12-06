use std::env;
use std::fs;
use std::io;
use std::path;
use std::sync::mpsc;
use std::thread;
use std::vec;

type Paths = vec::Vec<path::PathBuf>;
type Error = io::Result<()>;

type Msg = Option<path::PathBuf>;
type Sender = mpsc::Sender<Msg>;
type Receiver = mpsc::Receiver<Msg>;

fn list_repos(send : & Sender) -> Error {
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
    send.send(None.unwrap());

    Ok(())
}

fn main() -> Error {
    let (send, recv): (Sender, Receiver) = mpsc::channel();
    list_repos(& send)?;
    //println!("Hello, world!");

    Ok(())
}
