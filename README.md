A tool for working with multiple git repos easily.
This is an alternative to the mono-repo approach.

git-poly is written in rust.

# Features
- Very fast!
- Almost all operations a done in parallel
- Multi-platform (mac, linux, windows)
- Searching for git repos is done asynchronously, the moment we have found a git
  repo a new thread is created to process it.
- Regex based find and replace
- There's no config file.

# Git poly doesnt
- Track repo dependencies
- Specify how to organise your repos

# Similar/Related projects

You can use any of these tools along with 'git poly'.

## git slave
[home] (http://gitslave.sourceforge.net/gits-man-page.html#get_status_on_all_branches)
## google repo
[home] (https://gerrit.googlesource.com/git-repo)
## git submodule
[home] (https://git-scm.com/book/en/v2/Git-Tools-Submodules)
## git subtree
[home] (https://github.com/git/git/blob/master/contrib/subtree/git-subtree.txt)

# The Idea
Although putting all your code in a single repo simplifies a lot of things when
working across many codebases, it's difficult to do with the current git tools.

Having individual repositories per project (library/application) makes it fairly
straight forward to manage read/write permissions, continious builds and sandbox
repo history access.

The question this project is trying to answer is:
    Can we make it fairly straight forward way for a developer to work across multiple
    git repos at once.

git-poly tries to present multiple repos as if they are one repo for day to day operations.

The heart of this is git poly status. Which performs a 'git status' on all repos
and presents the results to make it appear as though you are in a single git repo.

Most of the commands are mirrors of git commands, but designed to work across
multiple projects, and give the effect of working on a single git repo.

The exception are three new commands:
- 'cmd' which will run whatever shell command you want across all repos in parallel. A bit like git submodule foreach
- 'go' which will run whatever git commands you want across all repos in parallel
- 'replace' which will perform a find and replace across all repos
- 'ls' which will list all your git repos

Most commands work with the '--path/-p' flag, which allows you to filter
the repos you are working on, using a regular expression that is matched against
the file path of the repo.

Those commands also work with the '--branch/-branch' flag, which allows you to filter
the repos you are working on, using a regular expression that is matched against
the branch the repo is currently tracking.

# Cloning

git poly doesn't use a config file. It just searches for folders containing
'.git' recursively from the current folder you are in.

The most straight forward way to clone is to use 'git clone' for each of
your repos. If you have a root repo that references other repos, then whatever
configuration you use to reference those other repos can be used to perform
a parallel cloning operation (a thread per clone).

This can be done by piping a list of repo urls in to 'git poly clone'.

Something like this:
```
cat config.yml | grep ".*\.git" | git poly clone
```

# Examples
## The simplest things
### Search for the word 'hello' in all your repos.
```
>> git p go grep hello
```

### Find and replace 'this' with 'that' in the files of every git repo we can find (using regex)
```
>> git p replace "this" "that"
```

### List all the git repos we can find
```
>> git p ls
```

## Additional things
### status
```
>> git p status
```

### add
```
>> git p add <filename>
```

# Build native
cargo build

# Build in docker [centos 7]
bash centos7/build.sh
