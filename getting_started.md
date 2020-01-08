# Getting started

## Download

|Platform|    Version    | Instruction Set |
|--------|---------------|-----------------|
|Linux   | Kernel 4.9.x  |    x86_64       |

## 101

### Clone

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


### Status

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

### Add and Commit

To add and commit those changes.

```bash
git p add ./libjpeg-turbo/libjpeg-turbo/jsimd.h
git p add ./openssl/openssl/ssl/ssl_rsa.c
```

or

```bash
git p add -u
```

then

```bash
git p commit -m "Ive made a change to some files"
```

