[![CI](https://github.com/towry/path-git-format/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/towry/path-git-format/actions/workflows/ci.yml) ![GitHub Release](https://img.shields.io/github/v/release/towry/path-git-format) ![GitHub top language](https://img.shields.io/github/languages/top/towry/path-git-format)

# path-git-format

Cli tool to format path(s) with git information.

## Usage

```
printf "$PWD" | path-git-format --format "{path}: {branch}" | fzf
```

### Use with zoxide query | [tldr; fish script](./extension/zoxide-path-git-format.fish)

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

## Snippets

See [extension/](./extension)

- `zoxide-path-git-format.fish`: fish functions to quickly jump to folder with git
  branch fuzzy match.
- `nvim-fzf-lua-zoxide-folders.lua`: In neovim, you can open fzf-lua folders
  picker with zoxide integration, to quickly change cwd like you does in
  terminal.

Video presentation:

https://github.com/towry/path-git-format/assets/8279858/30c5d166-9a30-4445-ac90-4593fb01fa6c
