//------------------------------------------------------------------------------
// Copyrite Luke Titley 2019
//------------------------------------------------------------------------------
use regex;
use std::env;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::vec;

use std::io::BufRead;
use std::io::Write;

use colored::*;

type Paths = vec::Vec<path::PathBuf>;
type Error = io::Result<()>;
type StatusMsg = (String, String, String);
type StatusSender = mpsc::Sender<StatusMsg>;
type StatusReceiver = mpsc::Receiver<StatusMsg>;
type PathMsg = Option<path::PathBuf>;
type PathSender = mpsc::Sender<PathMsg>;
type PathReceiver = mpsc::Receiver<PathMsg>;

//------------------------------------------------------------------------------
// Usage
//------------------------------------------------------------------------------
const USAGE: &str = "
USAGE:
    git poly [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -f, --filter <regex>   Filter repos using given expression

SUBCOMMANDS
    go [GIT COMMANDS]      Execute git commands in each repo
    ls                     List all the git repos discovered
    replace [FROM] [TO]    Find and replace all occurances of FROM with TO.
";

//------------------------------------------------------------------------------
fn usage() {
    println!("{0}", USAGE);
}

//------------------------------------------------------------------------------
fn argument_error(msg: &str) {
    println!("error: {0}\n{1}", msg, USAGE);
    std::process::exit(1);
}

//------------------------------------------------------------------------------
// list_repos
//------------------------------------------------------------------------------
fn list_repos(regex: &regex::Regex, send: &PathSender) -> Error {
    let mut current_dir = path::PathBuf::new();
    current_dir.push(".");

    let mut paths = Paths::new();

    paths.push(current_dir);

    // Walk over the directory
    while !paths.is_empty() {
        let path = paths.pop().unwrap();
        match fs::read_dir(path.clone()) {
            Ok(dir) => {
                for entry in dir {
                    let p = entry?.path();
                    if p.is_dir() {
                        let mut p_buf = p.to_path_buf();
                        let name = p.file_name().unwrap().to_str();
                        match name {
                            Some(".git") => {
                                // We've found a git repo, send it back
                                p_buf.pop();
                                let repo_path = p_buf.as_path();
                                if regex.is_match(repo_path.to_str().unwrap()) {
                                    send.send(Some(p_buf)).unwrap();
                                }
                            }
                            _ => {
                                paths.push(p_buf);
                            }
                        }
                    }
                }
            }
            Err(error) => {
                let mut stderr = std::io::stderr();
                writeln!(stderr, "{0} '{1}'", error, path.display()).unwrap();
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
    recv: PathReceiver,
}

//------------------------------------------------------------------------------
impl RepoIterator {
    fn new(regex: &regex::Regex) -> Self {
        let (send, recv): (PathSender, PathReceiver) = mpsc::channel();

        // Kick off the traversal thread. It's detached by default.
        let regex_copy = regex.clone();
        thread::spawn(move || list_repos(&regex_copy, &send).unwrap());

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
    writeln!(handle, "--{0}", "-".repeat(display.len()))?;
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
    // stderr
    if !output.is_empty() {
        let stderr = io::stderr();
        {
            let mut handle = stderr.lock();
            write_to_out(&mut handle, repo, output).unwrap();
        }
    }
}

//------------------------------------------------------------------------------
fn replace(regex: &regex::Regex, args_pos: usize) {
    let mut threads = Vec::new();

    let args: Vec<String> = env::args().collect();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        // Get hold of the from and to
        let from = args[args_pos + 1].clone();
        let to = args[args_pos + 2].clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {
            let from_exp = regex::Regex::new(&from).unwrap();

            let args = ["grep", "-l", from.as_str()];
            let output = process::Command::new("git")
                .args(&args)
                .current_dir(path.clone())
                .output()
                .unwrap();

            // stderr
            write_to_stderr(&path, &output.stderr);

            // perform the find and replace
            if !output.stdout.is_empty() {
                let mut replace_threads = Vec::new();
                let stdout = io::BufReader::new(&output.stdout as &[u8]);
                for line in stdout.lines() {
                    let file_path = path::Path::new(&path).join(line.unwrap());
                    let from_regex = from_exp.clone();
                    let to_regex = to.clone();
                    let replace_thread = thread::spawn(move || {
                        let mut output = Vec::<u8>::new();
                        {
                            let input =
                                fs::File::open(file_path.clone()).unwrap();
                            let buffered = io::BufReader::new(input);
                            for line in buffered.lines() {
                                let old_line = line.unwrap();
                                let new_line = from_regex.replace_all(
                                    &old_line as &str,
                                    &to_regex as &str,
                                );
                                writeln!(output, "{0}", new_line).unwrap();
                            }
                        }
                        let mut input = fs::File::create(file_path).unwrap();
                        input.write_all(&output).unwrap();
                    });

                    replace_threads.push(replace_thread);
                }

                // Wait for all the replace threads to finish
                for replace_thread in replace_threads {
                    replace_thread.join().unwrap();
                }
            }
        });

        threads.push(thread);
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
}

//------------------------------------------------------------------------------
fn go(regex: &regex::Regex, args_pos: usize) {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {
            let args: Vec<String> = env::args().collect();
            let output = process::Command::new("git")
                .args(&args[args_pos + 1..])
                .current_dir(path.clone())
                .output()
                .unwrap();

            // stdout/stderr
            write_to_stdout(&path, &output.stdout);
            write_to_stderr(&path, &output.stderr);
        });

        threads.push(thread);
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
}

//------------------------------------------------------------------------------
fn ls(regex: &regex::Regex) {
    for path in RepoIterator::new(regex) {
        let display = path.as_path().to_str().unwrap();
        println!("{0}", display);
    }
}

//------------------------------------------------------------------------------
fn get_branch_name(path: &path::PathBuf) -> String {
    let output = process::Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path.clone())
        .output()
        .unwrap();

    write_to_stderr(&path, &output.stderr);

    let stdout = io::BufReader::new(&output.stdout as &[u8]);
    let result: Vec<_> = stdout.lines().collect();

    if result.is_empty() {
        return "HEADLESS".to_string();
    }

    result[0].as_ref().unwrap().to_string()
}

//------------------------------------------------------------------------------
fn print_title(title: char) -> &'static str {
    match title {
        ' ' => {
            println!("Changes not staged for commit:");
            println!("  (use \"git add <file>...\" to include in what will be committed)");
            println!();
            return "red";
        }
        '?' => {
            println!("Untracked files:");
            println!("  (use \"git add <file>...\" to include in what will be committed)");
            println!();
            return "red";
        }
        'U' => {
            println!("You have unmerged paths.");
            println!("  (fix conflicts and run \"git commit\")");
            println!("  (use \"git merge --abort\" to abort the merge)");
            println!();
            println!("Unmerged paths:");
            println!("  (use \"git add <file>...\" to mark resolution)");
            return "red";
        }
        _ => {
            println!("Changes to be commited:");
            println!();
            return "green";
        }
    }
}

//------------------------------------------------------------------------------
fn status(regex: &regex::Regex) {
    let (send, recv): (StatusSender, StatusReceiver) = mpsc::channel();

    let splitter_def =
        regex::Regex::new(r"(UU| M|M |A | D|D |\?\?) (.*)").unwrap();

    let mut threads = Vec::new();
    for path in RepoIterator::new(regex) {
        let sender = send.clone();
        let splitter = splitter_def.clone();

        let thread = thread::spawn(move || {
            let branch_name = get_branch_name(&path);

            let args = ["status", "--porcelain"];
            let output = process::Command::new("git")
                .args(&args)
                .current_dir(path.clone())
                .output()
                .unwrap();

            write_to_stderr(&path, &output.stderr);

            let stdout = io::BufReader::new(&output.stdout as &[u8]);
            for line_result in stdout.lines() {
                let line = line_result.unwrap();
                let mut file_path = path::PathBuf::new();
                let split: Vec<_> =
                    splitter.captures_iter(line.as_str()).collect();

                if !split.is_empty() {
                    let status = &split[0][1];
                    let file = &split[0][2];
                    file_path.push(path.clone());
                    file_path.push(file);
                    sender
                        .send((
                            branch_name.clone(),
                            status.to_string(),
                            file_path.to_str().unwrap().to_string(),
                        ))
                        .unwrap();
                }
            }
        });

        threads.push(thread);
    }
    drop(send);

    // Wait for all the threads to finish
    for thread in threads {
        thread.join().unwrap();
    }

    // Store all the changes in a vector;
    let mut changes = Vec::from_iter(recv.iter());
    changes.sort();

    // Print the result
    if !changes.is_empty() {
        let mut branch_title = changes[0].0.clone();
        let mut title = changes[0].1.as_bytes()[0] as char;
        let mut color;
        println!("on branch {0}", branch_title.cyan());
        color = print_title(title);
        for change in changes {
            let (branch, status, path) = change;

            if branch_title != branch {
                branch_title = branch;
                title = '-';
                println!();
                println!("on branch {0}", branch_title.cyan());
            }
            let staged = status.as_bytes()[0] as char;
            if title != staged {
                if title != '-' {
                    println!();
                }
                title = staged;
                color = print_title(title);
            }

            match status.as_str() {
                "M " | " M" => {
                    print!("{0}", "        modified:   ".color(color))
                }
                "D " | " D" => {
                    print!("{0}", "        deleted:   ".color(color))
                }
                "A " => print!("{0}", "        new file:   ".color(color)),
                "UU" => print!("{0}", "        both modified:   ".color(color)),
                _ => print!("        "),
            }
            println!("{0}", path.color(color));
        }
        println!();
    }
}

//------------------------------------------------------------------------------
struct Flags {
    filter: regex::Regex,
}

//------------------------------------------------------------------------------
impl Flags {
    pub fn new() -> Self {
        Flags {
            filter: regex::Regex::new(r".*").unwrap(),
        }
    }
}

//------------------------------------------------------------------------------
fn main() -> Error {
    // The flags
    let mut flags = Flags::new();

    // Grab the arguments
    let env_args: Vec<String> = env::args().collect();

    if env_args.len() == 1 {
        argument_error("Not enough arguments");
    }

    // Args is argv without the executable name
    let args = &env_args[1..];

    // Execute the sub commands
    let mut skip: usize = 0;
    for index in 0..args.len() {
        if skip == 0 {
            let arg = &args[index];
            match arg.as_str() {
                // Flags
                "--help" | "-h" => {
                    usage();
                    break;
                }
                "--filter" | "-f" => {
                    if (index + 1) == args.len() {
                        argument_error(
                            "--filter requires an expression \
                             (ie --filter '.*')",
                        );
                    }
                    flags.filter =
                        regex::Regex::new(&(args[index + 1])).unwrap();
                    skip = 1;
                }
                // Sub-commands
                "go" => {
                    if index + 1 == args.len() {
                        argument_error("go requires at least one git command");
                    }
                    go(&flags.filter, index + 1);
                    break;
                }
                "ls" => {
                    ls(&flags.filter);
                    break;
                }
                "status" => {
                    status(&flags.filter);
                    break;
                }
                "replace" => {
                    if index + 2 >= args.len() {
                        argument_error(
                            "replace requires at least two arguments",
                        );
                    }
                    replace(&flags.filter, index + 1);
                    break;
                }
                _ => argument_error("argument not recognised"),
            }
        } else {
            skip -= 1;
        }
    }

    Ok(())
}
