use std::env;
use std::fs;
use std::io;
use std::path;
use std::vec;

type Paths = vec::Vec<path::PathBuf>;

type Error = io::Result<()>;

fn list_repos() -> Error {
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
                        p_buf.pop();
                        println!("{0}", p_buf.as_path().display());
                    }
                    _ => {
                        paths.push(p_buf);
                    }
                }
            }
        }
    }

    Ok(())
}

fn main() -> Error {
    list_repos()?;
    //println!("Hello, world!");

    Ok(())
}
