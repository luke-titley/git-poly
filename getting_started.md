# Getting started

## Download

|Platform|    Version    | Instruction Set |
|--------|---------------|-----------------|
|Linux   | Kernel 4.9.x  |    x86_64       |

## 101

### Cloning

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
grep --color=never --only-matching https.*\.git config.yml | git poly clone
```
