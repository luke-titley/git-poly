use std::path;
use std::env;
use std::result::Result;

type Error = Result<(), std::io::Error>;

fn list_repos() -> Error {
    let current_dir = env::current_dir()?.as_path();

    Ok(())
}

fn main() -> Error {
    list_repos()?;
    //println!("Hello, world!");

    Ok(())
}
