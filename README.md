A tool for working with multiple git repos easily.
This is an alternative to the mono-repo approach.

git-poly is written in rust, and intended to be fast.

Features:
- Searching for git repos is done asynchronously, the moment we have found a git
  repo a new thread is created to process it.


# Examples
## Run a git command across each repo
```
>> git poly go grep hello
```

## Find and replace this with that in the files of every git repo we can find
```
>> git poly replace "this" "that"
```

## List all the git repos we can find
```
>> git poly ls
```