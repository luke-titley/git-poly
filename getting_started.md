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
with, then you can take advantage of 'git poly clone' to download everything in
parallel.

Imagine for example you have a yml configuration file that looks like this.

```yml
repos:
    - https://github.com/openssl/openssl.git
    - https://github.com/libjpeg-turbo/libjpeg-turbo.git
```

Then to clone them all in parallel you can use grep.

```bash
grep --color=never --only-matching 'https.*\.git' config.yml | git poly clone
```

It's important to note that 'git poly clone' will maintain the folder path in
the url.

### Status

git poly will show you the status of all the files in the repos you have checked
out. It combines them to make it appear as if you are working in a single git
repo.

To perform a parallel 'git status' across all the repos and display the combined
results you use 'git poly status'

```bash
git poly status
```

For example imagine I edit both repos. Here I'm adding an additional comment to
a header file and a c file, one in libjpeg and one in openssl.

```bash
echo '// extra comment' >> libjpeg-turbo/libjpeg-turbo/jsimd.h
echo '// extra comment' >> openssl/openssl/ssl/ssl_rsa.c
```

Now if I run 'git poly status' this is the result I'll see.

```
on branch master
Changes not staged for commit:
  (use "git add <file>..." to include in what will be committed)

        modified:   ./libjpeg-turbo/libjpeg-turbo/jsimd.h
        modified:   ./openssl/openssl/ssl/ssl_rsa.c
```

Changes are grouped by the branches currently checked out. In the example above
both repos are on the master branch.


