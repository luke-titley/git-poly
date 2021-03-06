# Getting started

## Installation

Download the binary for your platform and place it somewhere within your
$PATH. The git-p statically links libc so the same binary can be used
across linux distributions.

git poly depends on git. So you must have git installed.

## Download

At the moment there is only a linux pre-built binary. Get in contact if
you can provide machines for building on osx or windows.

Alternatively, you can build it quite easily yourself using cargo from rust.

|Platform|    Version    | Instruction Set |        Download         |
|--------|---------------|-----------------|-------------------------|
|Linux   | Kernel 4.19.x  |    x86_64       | [here](linux/git-p) |

## What is git-poly ?

A tool to help developing across 50+ git repos easier. It's a simple,
light weight tool to parallelize working with many git repos.

### The Goal
Make working with many git repos feel like working in a single git repo for
most day to day operations, while still allowing those repos to be
completely seperate.

### Features
- Very fast!
- Almost all operations are done in parallel
- Multi-platform (mac, linux, windows)
- Searching for git repos is done asynchronously, the moment we have found a git
  repo a new thread is created to process it.
- Regex based find and replace
- There's no config/manifest file.
- Aligns very closely with git, only four more very straight forward commands
are added.

### Git poly doesnt
- Track repo dependencies
- Manage sub repo / parent repo histories
- Specify how to organise your repos

### Similar/Related projects

You can use these along with 'git poly'.

- [git slave](http://gitslave.sourceforge.net/gits-man-page.html#get_status_on_all_branches)
- [google repo](https://gerrit.googlesource.com/git-repo)
- [git submodule](https://git-scm.com/book/en/v2/Git-Tools-Submodules)
- [git subtree](https://github.com/git/git/blob/master/contrib/subtree/git-subtree.txt)

What's the point of git-poly. Why don't I just use one of the above?
git-poly fills a specific need. An unintrusive, simple way to make changes to
multiple repos at once.


# In what situations is git-poly useful?
If you have lots of little repos that depend on one and other, and you find
that you are often changing a few of them at a time when you develop a new
feature. You want the histories to remain independant, tags and releases to be
independant but you still need to change a few of them at a time.

# The Code

git-poly is written in rust. You can find the source code here:

- [github](https://github.com/luke-titley/git-poly)
- [gitlab](https://gitlab.com/luke.titley/git-poly)

## Getting Started

### clone

git poly doesnt have a manifest or configuration file, so it's up to you to
manage that.

The simplest way to clone the repos is git clone.

```bash
git clone https://github.com/openssl/openssl.git
git clone https://github.com/libjpeg-turbo/libjpeg-turbo.git
```

If you already have a setup in place that tracks the repos you are working
with, then you can take advantage of 'git p clone' to download everything in
parallel.

Imagine for example you have a yml configuration file that looks like this.

```yml
repos:
    - https://github.com/openssl/openssl.git
    - https://github.com/libjpeg-turbo/libjpeg-turbo.git
```

Then on linux to clone them all in parallel you can use grep.

```bash
grep --color=never --only-matching 'https.*\.git' config.yml | git p clone
```

It's important to note that 'git p clone' will maintain the folder path in
the url. This is different from git 'clone' which will clone into a single
folder.


### status

git poly will show you the status of all the files in the repos you have checked
out. It combines them to make it appear as if you are working in a single git
repo.

To perform a parallel 'git status' across all the repos and display the combined
results you use 'git p status'

```bash
git p status
```

For example imagine I edit both repos. Here I'm adding an additional comment to
a header file and a c file, one in libjpeg and one in openssl.

```bash
echo '// extra comment' >> libjpeg-turbo/libjpeg-turbo/jsimd.h
echo '// extra comment' >> openssl/openssl/ssl/ssl_rsa.c
```

Now if I run 'git p status' this is the result I'll see.

```
on branch master
Changes not staged for commit:
  (use "git add <file>..." to include in what will be committed)

        modified:   ./libjpeg-turbo/libjpeg-turbo/jsimd.h
        modified:   ./openssl/openssl/ssl/ssl_rsa.c
```

Changes are grouped by the branches currently checked out. In the example above
both repos are on the master branch.

If we switch libjpeg onto a branch called develop, git poly status will display
the results differently.


Here we're running the git command 'git checkout -b develop' on all the repos
which have 'libjpeg' in the path name. In this case, its only one repo.
```
git p --path libjpeg go checkout -b develop
```

Now if I run 'git p status', I get a different result.

```
on branch develop
Changes not staged for commit:
  (use "git add <file>..." to include in what will be committed)

        modified:   ./libjpeg-turbo/libjpeg-turbo/jsimd.h

on branch master
Changes not staged for commit:
  (use "git add <file>..." to include in what will be committed)

        modified:   ./openssl/openssl/ssl/ssl_rsa.c
```

### add 

To add those changes.

```bash
git p add ./libjpeg-turbo/libjpeg-turbo/jsimd.h
git p add ./openssl/openssl/ssl/ssl_rsa.c
```

or

```bash
git p add -u
```

### commit

To commit

```bash
git p commit -m "Ive made a change to some files"
```

### push
Finally, there isnt a first class 'push' subcommand. So you have to go via the
go subcommand.

```
git p go push
```

### grep

git poly can run 'git grep' in parallel over multiple repos. The benefit to
using 'git p grep' over 'git p go grep' is that the resulting file paths
are displayed with respect to the current directory.

```
git p grep hel
```

### ls-files

Similarly to 'git p grep', there is first class support for 'git ls-files', with
resulting file paths correctly adjusted.

```
git p ls-files
```

### mv

This sub command is a utility for moving files from one repo to another.

```
git p mv ./openssl/openssl/ssl/ssl_rsa.c ./libjpeg-turbo/libjpeg-turbo/ssl_rsa.c
```

### reset

You can reset everything you have staged with 'git p reset'.

```
git p reset
```

Then if you want to throw away everything that's changed.

```
git p go checkout .
```

### replace

You can perform a parallel find and replace, using 'git p replace'.
This will run a thread per repo and then a thread per file.

```
git p replace cat dog
```

### go

This runs a git command across the given repos in parallel.

For example
```
git p go add -u
```

will stage any unstaged changes in any of the repos you are working in.

### cmd

The same as 'go' command, only executes shell commands.

