A tool for working with multiple git repos easily.
This is an alternative to the mono-repo approach.

git-poly is written in rust, and intended to be fast.

Features:
- Searching for git repos is done asynchronously, the moment we have found a git
  repo a new thread is created to process it.


Examples
```
>> git poly go grep hello
```

```
>> git poly replace "this" "that"
```

```
>> git poly ls
```