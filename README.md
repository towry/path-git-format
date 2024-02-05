# path-git-format

Cli tool to format path(s) with git information.

## Usage

```
printf "$PWD" | path-git-format --format "{path}: {branch}" | fzf
```

### Use with zoxide query

[zoxide](https://github.com/ajeetdsouza/zoxide)'s `query` command returns list of paths:

```
zoxide query --list --score
```

output:

```txt
100 /Users/towry/projects/a
80  /Users/towry/projects/b
01  /Users/towry/projects/c
```

We can use `path-git-format` to format those paths with git branch information.

```bash
zoxide query --list --score | path-git-format --nth 1 --format "{path}: {branch}" | fzf
```

So you can use it with `fzf` to search paths along with git branch.
