[![CI](https://github.com/towry/path-git-format/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/towry/path-git-format/actions/workflows/ci.yml) ![GitHub Release](https://img.shields.io/github/v/release/towry/path-git-format) ![GitHub top language](https://img.shields.io/github/languages/top/towry/path-git-format)


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

## Install

### Install binary release

1. [Visit the Release page to download the appropriate tarball for your system](https://github.com/towry/path-git-format/releases)
2. Extract the binary file and put it under your `$PATH` like `/user/local/bin`.

### Prerequisites

- cargo
- git

### Install with cargo

```
cargo install --git https://github.com/towry/path-git-format
```

### Build and Install from source

```bash
git clone --depth=1 git@github.com:towry/path-git-format.git
cd path-git-format
# will install `path-git-format` into `/usr/local/bin`
make install
```

## Snippet

Fish script to make `jump` works like zoxide but with git branch in paths, put
this in your fish `config.fish`.

```fish
function jump --description "Zoxide jump with git branch in path"
    set -l query "$argv"
    set result (zoxide query --list --exclude $PWD | path-git-format --filter --no-bare -f"{path} [{branch}]" | awk -v home="$HOME" '{gsub(home, "~", $1); print $0}' | fzf --tiebreak=index -1 -0 --query="$query")
    if test -n "$result"
        set directory (echo $result | awk -F' ' '{print $1}' | awk -F'[' '{print $1}')
        if test "$_ZO_ECHO" = "1"
            echo "$directory"
        end
        eval cd $directory
    end
end


function jump_first --description "Zoxide jump(first) with git branch in path"
    set -l query "$argv"
    set result (zoxide query --list --exclude $PWD | path-git-format --filter --no-bare -f"{path} [{branch}]" | awk -v home="$HOME" '{gsub(home, "~", $1); print $0}' | fzf --tiebreak=index --filter="$query" | head -n 1)
    if test -n "$result"
        set directory (echo $result | awk -F' ' '{print $1}' | awk -F'[' '{print $1}')
        if test "$_ZO_ECHO" = "1"
            echo "$directory"
        end
        eval cd $directory
    end
end

```

Video presentation:

https://github.com/towry/path-git-format/assets/8279858/30c5d166-9a30-4445-ac90-4593fb01fa6c


