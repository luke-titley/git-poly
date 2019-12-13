A tool for working with multiple git repos easily.
This is an alternative to the mono-repo approach.

git-poly is written in rust.

# Features
- Very fast!
- Multi-platform
- Searching for git repos is done asynchronously, the moment we have found a git
  repo a new thread is created to process it.
- Regex based find and replace

# Prior-art
## git slave [home] (http://gitslave.sourceforge.net/gits-man-page.html#get_status_on_all_branches)

# The Idea
Although putting all your code in a single repo simplifies a lot of things when
working across many codebases, it's difficult to do with the current git tools.

Having individual repositories per project (library/application) makes it fairly
straight forward to manage read/write permissions, continious builds and sandbox
file history access.

git doesnt scale so well if you put your entire codebase under a single git repo.
Although perforce manages permissions at the revision control level, git doesnt.

The question this project is trying to answer is:
    Can we make it fairly straight forward way for a developer to work across multiple
    git repos at once.

git-poly tries to present multiple repos as if they are one repo for the 60% of operations
a developer does (add, commit, reset, status, checkout, branch, pull).
Most of the commands are mirrors of git commands, but designed to work across projects.

The exceptions to this are three new commands:
- 'go' which will run whatever git commands you want across all repos
- 'replace' which will perform a find and replace across all repos
- 'ls' which will list all your git repos

Those commands all work with the '--filter/-f' flag, which allows you to filter
the repos you are working on, using a regular expression that is matched against
the file path of the repo.

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

## The smart things
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
