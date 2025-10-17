use clap::Parser;
use git2::Repository;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use strfmt::strfmt;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_stream::wrappers::LinesStream;
#[allow(unused_imports)]
use tokio_stream::StreamExt;

#[derive(Debug)]
struct CliOptions {
    format: Option<String>,
    nth: usize,
    filter: bool,
    no_bare: bool,
}

#[derive(Debug)]
struct VcsInfo<'a> {
    path_index: usize,
    segments: Vec<&'a str>,
    branch: Option<String>,
}

impl<'a> VcsInfo<'a> {
    fn new(segments: Vec<&'a str>, path_index: usize) -> Self {
        VcsInfo {
            path_index,
            segments,
            branch: None,
        }
    }

    fn path_str(&self) -> Option<&str> {
        self.segments.get(self.path_index).copied()
    }

    async fn update_branch(&mut self, opts: &CliOptions) {
        let path = self.path_str();
        if path.is_none() {
            return;
        }
        let path = PathBuf::from(path.unwrap());

        // First, try to detect if this is a jujutsu repository
        let jj_dir = path.join(".jj");
        if jj_dir.exists() && jj_dir.is_dir() {
            // This is a jujutsu repository
            if let Some(bookmarks) = get_jj_bookmarks(&path) {
                self.branch = Some(bookmarks);
                return;
            }
        }

        // Fall back to git detection
        let repo = Repository::open(&path).ok();
        let Some(repo) = repo else {
            return;
        };

        if opts.no_bare && repo.is_bare() {
            return;
        }

        // filter out useless folder at <repo>/worktrees/<folder>
        if Some(path.as_path()) != repo.workdir() {
            return;
        }

        let branch = repo
            .head()
            .map(|head| head.shorthand().map(|s| s.to_owned()))
            .ok()
            .flatten();
        self.branch = branch
    }
}

// Get bookmarks from a jujutsu repository that point to commits in the working copy's ancestry
fn get_jj_bookmarks(path: &PathBuf) -> Option<String> {
    // First, get the current working copy commit using `jj log -r @`
    let wc_output = Command::new("jj")
        .args(["log", "-r", "@", "--no-graph", "-T", "commit_id"])
        .current_dir(path)
        .output()
        .ok()?;

    if !wc_output.status.success() {
        return None;
    }

    let wc_commit_id = String::from_utf8_lossy(&wc_output.stdout).trim().to_owned();
    if wc_commit_id.is_empty() {
        return None;
    }

    // Get all local bookmarks (not remotes)
    let bookmarks_output = Command::new("jj")
        .args(["bookmark", "list"])
        .current_dir(path)
        .output()
        .ok()?;

    if !bookmarks_output.status.success() {
        return None;
    }

    let bookmarks_text = String::from_utf8_lossy(&bookmarks_output.stdout);
    let mut matching_bookmarks = Vec::new();

    // Parse bookmark list to find bookmarks pointing to commits in working copy's ancestry
    // Format: "bookmark_name: commit_id description"
    for line in bookmarks_text.lines() {
        // Skip lines that are indented (remote tracking bookmarks start with spaces)
        if line.starts_with(' ') {
            continue;
        }

        if let Some(colon_pos) = line.find(':') {
            let bookmark_name = line[..colon_pos].trim();

            // Extract commit ID from the line (it's after the colon and space)
            let rest = line[colon_pos + 1..].trim();
            if let Some(space_pos) = rest.find(' ') {
                let commit_id = &rest[..space_pos];

                // Check if this commit is the working copy (compare prefixes since bookmark list shows short IDs)
                if wc_commit_id.starts_with(commit_id) {
                    matching_bookmarks.push(bookmark_name.to_owned());
                } else {
                    // Check if this commit is an ancestor of working copy using revset
                    // The revset "commit_id::@" means commits from commit_id to @ (working copy)
                    let ancestor_check = Command::new("jj")
                        .args([
                            "log",
                            "-r",
                            &format!("{}::@", commit_id),
                            "--no-graph",
                            "-T",
                            "commit_id",
                        ])
                        .current_dir(path)
                        .output()
                        .ok()?;

                    if ancestor_check.status.success() {
                        let output_text = String::from_utf8_lossy(&ancestor_check.stdout);
                        let output_len = output_text.trim().len();
                        // Each commit ID is 40 chars. If we have >= 40 chars and the revset succeeded,
                        // it means the bookmark is in the ancestry (including @)
                        if output_len >= 40 {
                            matching_bookmarks.push(bookmark_name.to_owned());
                        }
                    }
                }
            }
        }
    }

    if matching_bookmarks.is_empty() {
        None
    } else {
        Some(matching_bookmarks.join(", "))
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // the format in each line.
    #[arg(
        short = 'f',
        long,
        help = "Format the output with {path}, {branch} placeholder"
    )]
    format: Option<String>,
    // parse input
    #[arg(
        short = 'n',
        long,
        help = "nth segment of line is the path, line segments separated by whitespace"
    )]
    nth: Option<usize>,
    // ignore paths that not under git repo
    #[arg(long, action, help = "Filter out non git repo path")]
    filter: bool,
    #[arg(long, action, help = "Filter out bare repo")]
    no_bare: bool,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let opts = CliOptions {
        format: args.format,
        nth: args.nth.map_or(0, |x| x),
        filter: args.filter,
        no_bare: args.no_bare,
    };

    read_io_paths(&opts).await
}

// iterate the stdin lines: https://doc.rust-lang.org/std/io/struct.Stdin.html#method.lines
async fn read_io_paths(opts: &CliOptions) -> io::Result<()> {
    let mut writter = io::stdout();

    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let lines = reader.lines();
    let mut lines_stream = LinesStream::new(lines);

    while let Some(v) = lines_stream.next().await {
        let line = v?;
        let Some(result_line) = process_line(opts, &line).await else {
            continue;
        };

        // Handle BrokenPipe errors gracefully - this happens when the downstream
        // process closes the pipe early (e.g., head -n 1, fzf)
        if let Err(e) = writter.write_all(result_line.as_bytes()).await {
            if e.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e);
        }
        if let Err(e) = writter.write_all(b"\n").await {
            if e.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }
            return Err(e);
        }
    }

    // Ignore BrokenPipe on shutdown as well
    if let Err(e) = writter.shutdown().await {
        if e.kind() == io::ErrorKind::BrokenPipe {
            return Ok(());
        }
        return Err(e);
    }

    Ok(())
}

async fn process_line(opts: &CliOptions, line: &str) -> Option<String> {
    // trim line .then separate line by space
    let segments = line.trim().split(' ').collect::<Vec<&str>>();
    let mut vcsinfo = VcsInfo::new(segments, opts.nth);
    vcsinfo.update_branch(opts).await;

    if vcsinfo.branch.is_none() && opts.filter {
        return None;
    }

    if vcsinfo.branch.is_none() {
        return Some(vcsinfo.path_str().unwrap_or("").to_owned());
    }

    let mut vars = HashMap::<String, &str>::new();
    vars.insert("path".to_owned(), vcsinfo.path_str().unwrap_or(""));
    vars.insert("branch".to_owned(), vcsinfo.branch.as_deref().unwrap_or(""));

    let fmt = opts.format.as_deref().unwrap_or("{path} {branch}");
    strfmt(fmt, &vars).ok()
}
