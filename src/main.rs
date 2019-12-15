//------------------------------------------------------------------------------
// Copyrite Luke Titley 2019
//------------------------------------------------------------------------------
use regex;
use std::env;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::str::FromStr;
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
type BranchRegex = Option<regex::Regex>;

//------------------------------------------------------------------------------
// Usage
//------------------------------------------------------------------------------
const USAGE: &str = "
USAGE:
    git poly [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -p, --path <regex>        Filter by repo file path using given expression
    -b, --branch <regex>      Filter by current branch using given expression

SUBCOMMANDS
    go [git command]          Execute git commands in each repo
    cmd [comands]             Execute one or more shell commands in each repo
    ls                        List all the git repos discovered

    add [-u] [<pathspec>...]  Add file contents to the index of it's repo
    commit [-m] <message>     Record changes to the repository
    grep <pattern>            Print lines matching a pattern
    ls-files                  Show information about files in the index and the working tree
    mv <from> <to>            Move or rename a file, a directory, or a symlink
    status                    Show the merged working tree status of all the repos

    replace <from> <to>       Find and replace all occurances of FROM with TO
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

    writeln!(handle, "{0}", display.cyan())?;
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
fn filter_branch(expression : &regex::Regex, path: &path::PathBuf) -> bool {
    let branch_name = get_branch_name(path);
    return expression.is_match(branch_name.as_str());
}

//------------------------------------------------------------------------------
fn replace(regex: &regex::Regex, branch_regex: &BranchRegex, args_pos: usize) {
    let mut threads = Vec::new();

    let args: Vec<String> = env::args().collect();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        // Get hold of the from and to
        let from = args[args_pos + 1].clone();
        let to = args[args_pos + 2].clone();
        let branch_filter = branch_regex.clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {

            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

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
fn go(path_regex: &regex::Regex, branch_regex: &BranchRegex, args_pos: usize) {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(path_regex) {
        let branch_filter = branch_regex.clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {

            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

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
fn cmd(regex: &regex::Regex, branch_regex: &BranchRegex, args_pos: usize) {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        let branch_filter = branch_regex.clone();

        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {

            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

            let args: Vec<String> = env::args().collect();
            let args_ref = &args[args_pos + 1..];
            let output = process::Command::new(args_ref[0].clone())
                .args(&args_ref[1..])
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
fn add_changed(regex: &regex::Regex) {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        // Execute a new thread for processing this result
        let thread = thread::spawn(move || {
            let args = ["add", "-u"];
            let output = process::Command::new("git")
                .args(&args)
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
fn relative_to_repo(path : & mut path::PathBuf) -> Option<(path::PathBuf, String)> {

    for parent in path.ancestors() {
        if ! parent.as_os_str().is_empty() {
            let mut repo = path::PathBuf::new(); // TODO LT: Wanted to keep this around but need nightly to use 'clear'.
            repo.push(parent);
            repo.pop();
            repo.push(".git");

            if repo.exists() {
                repo.pop();
                let relative_path = path.as_path().strip_prefix(repo.as_path()).unwrap().to_str().unwrap();
                return Some(( repo, relative_path.to_string() ));
            }
        }
    }

    None
}

//------------------------------------------------------------------------------
fn add_entry(path : & mut path::PathBuf) {
    if let Some((repo, relative_path)) = relative_to_repo(path) {

        let args = ["add", relative_path.as_str()];
        let output = process::Command::new("git")
            .args(&args)
            .current_dir(repo.clone())
            .output()
            .unwrap();

        // stdout/stderr
        write_to_stdout(&repo, &output.stdout);
        write_to_stderr(&repo, &output.stderr);
    }
}

//------------------------------------------------------------------------------
fn reset_entry(path : & mut path::PathBuf) {
    if let Some((repo, relative_path)) = relative_to_repo(path) {

        let args = ["reset", relative_path.as_str()];
        let output = process::Command::new("git")
            .args(&args)
            .current_dir(repo.clone())
            .output()
            .unwrap();

        // stdout/stderr
        write_to_stdout(&repo, &output.stdout);
        write_to_stderr(&repo, &output.stderr);
    }
}

//------------------------------------------------------------------------------
fn ls_files(regex: &regex::Regex, branch_regex: &BranchRegex) {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        let branch_filter = branch_regex.clone();

        threads.push(thread::spawn(move || {

            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

            let output = process::Command::new("git")
                .args(&["ls-files"])
                .current_dir(path.clone())
                .output()
                .unwrap();

            write_to_stderr(&path, &output.stderr);

            let outstream = io::stdout();
            {
                let _handle = outstream.lock();
                let stdout = io::BufReader::new(&output.stdout as &[u8]);
                let flat_path = path.as_path().join( path::Path::new("") );
                for line in stdout.lines() {
                    print!("{0}", flat_path.display());
                    println!("{0}", line.unwrap());
                }
            }
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
}

//------------------------------------------------------------------------------
fn grep(regex: &regex::Regex, branch_regex: &BranchRegex, expression : &str) {
    let mut threads = Vec::new();

    // Loop through the results of what the walker is outputting
    for path in RepoIterator::new(regex) {
        let expr = expression.to_string();
        let branch_filter = branch_regex.clone();

        threads.push(thread::spawn(move || {

            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

            let output = process::Command::new("git")
                .args(&["grep", expr.as_str()])
                .current_dir(path.clone())
                .output()
                .unwrap();

            write_to_stderr(&path, &output.stderr);

            let outstream = io::stdout();
            {
                let _handle = outstream.lock();
                let stdout = io::BufReader::new(&output.stdout as &[u8]);
                let flat_path = path.as_path().join( path::Path::new("") );
                for line in stdout.lines() {
                    print!("{0}", flat_path.display());
                    println!("{0}", line.unwrap());
                }
            }
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
}

//------------------------------------------------------------------------------
fn add(regex: &regex::Regex, args_pos: usize) {
    let args: Vec<String> = env::args().collect();

    let mut minus_u = false;
    for arg in args_pos+1..args.len() {
        match args[arg].as_str() {
            "-u" => {
                if ! minus_u {
                    minus_u = true;
                    add_changed(regex)
                }
            },
            file_path => {
                let mut path = path::PathBuf::from(file_path);
                add_entry(& mut path);
            }
        }
    }
}

//------------------------------------------------------------------------------
fn reset_all(regex: &regex::Regex, branch_regex: &BranchRegex) {

    // Filtered traversal
    if let Some(pattern) = branch_regex {
        for path in RepoIterator::new(regex) {
            if filter_branch(&pattern, &path) {
                let display = path.as_path().to_str().unwrap();
                println!("Resetting {0}", display);
            }
        }

    // Unfiltered traversal
    } else {
        for path in RepoIterator::new(regex) {
            let display = path.as_path().to_str().unwrap();
            println!("Resetting {0}", display);
        }
    }
}

//------------------------------------------------------------------------------
fn reset(regex: &regex::Regex, branch_regex: &BranchRegex, args_pos: usize) {
    let args: Vec<String> = env::args().collect();

    // Reset individual files
    if args.len() - args_pos+1 > 0 {
        for arg in args_pos+1..args.len() {
                let mut path = path::PathBuf::from(args[arg].clone());
                reset_entry(& mut path);
        }

    // Reset all the repos
    } else {
        reset_all(regex, branch_regex);
    }
}

//------------------------------------------------------------------------------
fn ls(regex: &regex::Regex, branch_regex: &BranchRegex) {

    // Filtered traversal
    if let Some(pattern) = branch_regex {
        for path in RepoIterator::new(regex) {
            if filter_branch(&pattern, &path) {
                let display = path.as_path().to_str().unwrap();
                println!("{0}", display);
            }
        }

    // Unfiltered traversal
    } else {
        for path in RepoIterator::new(regex) {
            let display = path.as_path().to_str().unwrap();
            println!("{0}", display);
        }
    }
}

//------------------------------------------------------------------------------
fn commit(regex: &regex::Regex, branch_regex: &BranchRegex, msg : &str) {
    let mut threads = Vec::new();

    let changes =
        regex::Regex::new(r"^(M|A|D) .*").unwrap();

    for path in RepoIterator::new(regex) {
        let message = String::from_str(msg).unwrap();
        let c = changes.clone();
        let branch_filter = branch_regex.clone();

        threads.push( thread::spawn(move || {

            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

            let args = ["status", "--porcelain"];
            let output = process::Command::new("git")
                .args(&args)
                .current_dir(path.clone())
                .output()
                .unwrap();

            write_to_stderr(&path, &output.stderr);

            // Search for modifications
            let stdout = io::BufReader::new(&output.stdout as &[u8]);
            let mut lines = stdout.lines();
            let has_modifications = {
                loop {
                    if let Some(result) = lines.next() {
                        let line = result.unwrap();
                        if c.is_match(line.as_str()) {
                            break true;
                        }
                    } else {
                        break false;
                    }
                }
            };

            // If we have modifications then do a commit
            if has_modifications {
                let output = process::Command::new("git")
                    .args(&["commit", "-m", message.as_str()])
                    .current_dir(path.clone())
                    .output()
                    .unwrap();

                write_to_stderr(&path, &output.stderr);
                write_to_stdout(&path, &output.stdout);
            }
        }));
    }

    // Wait for all the threads to finish
    for thread in threads {
        thread.join().unwrap();
    }
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
fn status(regex: &regex::Regex, branch_regex: &BranchRegex) {
    let (send, recv): (StatusSender, StatusReceiver) = mpsc::channel();

    let splitter_def =
        regex::Regex::new(r"(UU| M|M |MM|A | D|D |\?\?) (.*)").unwrap();

    let mut threads = Vec::new();
    for path in RepoIterator::new(regex) {
        let sender = send.clone();
        let splitter = splitter_def.clone();
        let branch_filter = branch_regex.clone();

        let thread = thread::spawn(move || {
            
            // Filter based on branch name
            if let Some(pattern) = branch_filter {
                if !filter_branch(&pattern, &path) {
                    return;
                }
            }

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
    changes.sort(); // TODO LT: Use sort_by with a comparison function that orders the output similarly to git

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
fn mv(from : &str, to: &str) {
    let mut from_path = path::PathBuf::new();
    let mut to_path = path::PathBuf::new();

    from_path.push(from);
    to_path.push(to);

    if let Some((from_repo, from_rel)) = relative_to_repo(& mut from_path) {
        if let Some((to_repo, to_rel)) = relative_to_repo(& mut to_path) {

            if from_path.exists() {

                // Remove the destionation if it exists
                if to_path.exists() {
                    let output = process::Command::new("git")
                        .args(&["rm", "-rf", to_rel.as_str()])
                        .current_dir(to_repo.clone())
                        .output()
                        .unwrap();

                    write_to_stderr(&to_repo, &output.stderr);
                }

                // Move the file
                fs::rename(&from_path, &to_path);
                /*
                if from_path.is_dir() {
                    fs_extra::dir::copy(&from_path, &to_path, &fs_extra::dir::CopyOptions::new()).unwrap();
                } else {
                    fs::copy(&from_path, &to_path).unwrap();
                }
                */

                // Remove the old file or folder
                {
                    let output = process::Command::new("git")
                        .args(&["rm", "-rf", from_rel.as_str()])
                        .current_dir(from_repo.clone())
                        .output()
                        .unwrap();
                    write_to_stderr(&to_repo, &output.stderr);
                }

                // Add the newfile or folder
                {
                    let output = process::Command::new("git")
                        .args(&["add", to_rel.as_str()])
                        .current_dir(to_repo.clone())
                        .output()
                        .unwrap();
                    write_to_stderr(&to_repo, &output.stderr);
                }
            }
        }
    }
}

//------------------------------------------------------------------------------
struct Flags {
    path: regex::Regex,
    branch: BranchRegex,
}

//------------------------------------------------------------------------------
impl Flags {
    pub fn new() -> Self {
        Flags {
            path: regex::Regex::new(r".*").unwrap(),
            branch: None,
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
                "--path" | "-p" => {
                    if (index + 1) == args.len() {
                        argument_error(
                            "--path requires an expression \
                             (ie --path '.*')",
                        );
                    }
                    flags.path =
                        regex::Regex::new(&(args[index + 1])).unwrap();
                    skip = 1;
                }
                "--branch" | "-b" => {
                    if (index + 1) == args.len() {
                        argument_error(
                            "--branch requires an expression \
                             (ie --branch 'feature/foo.*')",
                        );
                    }
                    flags.branch =
                        Some(regex::Regex::new(&(args[index + 1])).unwrap());
                    skip = 1;
                }
                // Sub-commands
                "go" => {
                    if index + 1 == args.len() {
                        argument_error("go requires at least one git command");
                    }
                    go(&flags.path, &flags.branch, index + 1);
                    break;
                }
                "cmd" => {
                    if index + 1 == args.len() {
                        argument_error("cmd requires at least one shell command");
                    }
                    cmd(&flags.path, &flags.branch, index + 1);
                    break;
                }
                "add" => {
                    if index + 1 == args.len() {

                        let error =
"Nothing specified, nothing added.
Maybe you wanted to say 'git add .'?";
                        argument_error(error);
                    }
                    add(&flags.path, index + 1);
                    break;
                }
                "grep" => {
                    if index + 1 == args.len() {
                        argument_error("Please provide the expression you would like to grep for");
                    }
                    grep(&flags.path, &flags.branch, args[index+1].as_str());
                    break;
                }
                "ls-files" => {
                    ls_files(&flags.path, &flags.branch);
                    break;
                }
                "ls" => {
                    ls(&flags.path, &flags.branch);
                    break;
                }
                "commit" => {
                    if index + 2 >= args.len() {
                        argument_error(
                            "commit requires at least arguments -m and a message",
                        );
                    }

                    if args[index + 1] != "-m" {
                        argument_error(
                            "commit requires -m",
                        );
                    }

                    commit(&flags.path, &flags.branch, args[index + 2].as_str());
                    break;
                }
                "reset" => {
                    reset(&flags.path, &flags.branch, index);
                    break;
                }
                "status" => {
                    status(&flags.path, &flags.branch);
                    break;
                }
                "mv" => {
                    if index + 2 >= args.len() {
                        argument_error(
                            "mv requires a source and a dest",
                        );
                    }
                    mv(&args[index + 1], &args[index+2]);
                    break;
                }
                "replace" => {
                    if index + 2 >= args.len() {
                        argument_error(
                            "replace requires at least two arguments",
                        );
                    }
                    replace(&flags.path, &flags.branch, index + 1);
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
