# Testing Jujutsu (jj) Support

This guide explains how to test the jujutsu repository bookmark detection feature in `path-git-format`.

## Prerequisites

1. Install jujutsu CLI:
   ```bash
   cargo install --git https://github.com/jj-vcs/jj --locked jj-cli
   ```

2. Build path-git-format:
   ```bash
   cd /path/to/path-git-format
   cargo build --release
   ```

## Setting up a Demo Repository

### Create a Basic jj Repository

```bash
# Create a new directory for testing
mkdir jj-demo
cd jj-demo

# Initialize a jj repository
jj git init --git-repo=.

# Configure user (required for commits)
jj config set --user user.name "Test User"
jj config set --user user.email "test@example.com"

# Create initial commit
echo "Initial content" > README.md
jj describe -m "Initial commit"

# Create a bookmark for the initial commit
jj bookmark create main -r @

# Create a second commit
jj new -m "Second commit"
echo "More content" >> README.md

# Create a bookmark for the second commit
jj bookmark create feature -r @

# View the commit history
jj log
```

Expected output structure:
```
@  <commit-id> feature
│  Second commit
○  <commit-id> main
│  Initial commit
◆  root()
```

## Testing the Implementation

### Test 1: Jujutsu Repository with Bookmarks

From the jj-demo directory:

```bash
printf "$(pwd)" | /path/to/path-git-format/target/release/path-git-format --format "{path}: {branch}"
```

**Expected output:**
```
/path/to/jj-demo: feature, main
```

This shows both the `feature` bookmark (at working copy) and `main` bookmark (parent in first-parent ancestry).

### Test 2: Multiple Paths

Test with mixed repository types:

```bash
printf "/path/to/jj-demo\n/path/to/git-repo\n/tmp" | path-git-format --format "{path}: {branch}"
```

**Expected output:**
```
/path/to/jj-demo: feature, main
/path/to/git-repo: <git-branch-name>
/tmp
```

### Test 3: With Filter Flag

Filter out non-VCS paths:

```bash
printf "/path/to/jj-demo\n/tmp" | path-git-format --format "{path}: {branch}" --filter
```

**Expected output:**
```
/path/to/jj-demo: feature, main
```

## Testing Merge Commits

To test first-parent traversal with merge commits:

```bash
cd jj-demo

# Create a branch
jj new main -m "Branch commit"
jj bookmark create branch-a -r @

# Go back to feature and merge
jj new feature
jj merge branch-a -m "Merge branch-a into feature"

# The bookmark list should still only show bookmarks in first-parent ancestry
printf "$(pwd)" | path-git-format --format "{path}: {branch}"
```

The implementation follows only the first parent chain, so merged branch bookmarks won't appear unless they're in the first-parent ancestry.

## Commit Lookup Limit

The implementation limits traversal to the last 10 commits by default to prevent performance issues in large repositories. This means:
- Only bookmarks within 10 commits of the working copy (following first-parent) are returned
- This is sufficient for most use cases where you want to see "current" bookmarks
- Deeper history bookmarks are ignored for performance

## Troubleshooting

### "Failed to load workspace" error
- Ensure you're in a valid jj repository (has `.jj` directory)
- Check that jj is properly installed and configured
- Verify user.name and user.email are set in jj config

### Empty bookmark list
- Verify bookmarks exist: `jj bookmark list`
- Check that bookmarks are in the working copy's first-parent ancestry
- Ensure the repository has commits (not just root)

### Git fallback
If the tool returns git branch names instead of jj bookmarks:
- Confirm `.jj` directory exists in the repository root
- Check that the path points to the repository root, not a subdirectory
- Verify jj-lib can load the repository (no config errors)

## Cleanup

```bash
# Remove the demo repository
cd ..
rm -rf jj-demo
```
