use clap::Parser;
use git2::Repository;
use std::collections::HashMap;
use std::path::PathBuf;
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

// Get bookmarks from a jujutsu repository following first-parent ancestry from working copy
fn get_jj_bookmarks(path: &std::path::Path) -> Option<String> {
    use indexmap::IndexSet;
    use jj_lib::local_working_copy::LocalWorkingCopyFactory;
    use jj_lib::settings::UserSettings;
    use jj_lib::workspace::Workspace;
    use std::collections::HashSet;

    // Load workspace with minimal config
    let store_factories = jj_lib::repo::StoreFactories::default();
    let mut working_copy_factories: jj_lib::workspace::WorkingCopyFactories = Default::default();
    working_copy_factories.insert("local".to_string(), Box::new(LocalWorkingCopyFactory {}));

    // Create minimal user settings
    let config = create_minimal_jj_config()?;
    let settings = UserSettings::from_config(config).ok()?;

    // Load the workspace and repo
    let workspace =
        Workspace::load(&settings, path, &store_factories, &working_copy_factories).ok()?;
    let repo = workspace.repo_loader().load_at_head().ok()?;

    // Get the working copy commit ID as the starting point
    let wc_commit_id = repo.view().get_wc_commit_id(workspace.workspace_name())?;

    // Traverse commits following first-parent chain
    let mut visited = HashSet::new();
    let mut bookmark_names = IndexSet::new();

    // Start traversal from working copy commit
    traverse_first_parent(&repo, wc_commit_id, &mut visited, &mut bookmark_names);

    if bookmark_names.is_empty() {
        None
    } else {
        // Return sorted, deduplicated bookmark names
        Some(bookmark_names.into_iter().collect::<Vec<_>>().join(", "))
    }
}

// Helper to traverse commits following first-parent chain
// Limited to last 10 commits for performance
fn traverse_first_parent(
    repo: &jj_lib::repo::ReadonlyRepo,
    start_id: &jj_lib::backend::CommitId,
    visited: &mut std::collections::HashSet<jj_lib::backend::CommitId>,
    bookmark_names: &mut indexmap::IndexSet<String>,
) {
    use jj_lib::repo::Repo;

    const MAX_COMMITS: usize = 10;
    let mut current_id = start_id.clone();
    let mut commit_count = 0;

    loop {
        // Stop if we've checked enough commits
        if commit_count >= MAX_COMMITS {
            break;
        }

        // Skip if already visited
        if visited.contains(&current_id) {
            break;
        }
        visited.insert(current_id.clone());
        commit_count += 1;

        // Get local bookmarks for this commit
        let view = repo.view();
        for (ref_name, _ref_target) in view.local_bookmarks_for_commit(&current_id) {
            bookmark_names.insert(ref_name.as_str().to_owned());
        }

        // Load commit and get first parent
        let commit = match repo.store().get_commit(&current_id) {
            Ok(c) => c,
            Err(_) => break,
        };

        // Follow first parent only (to handle merges linearly)
        match commit.parent_ids().first() {
            Some(parent_id) => {
                current_id = parent_id.clone();
            }
            None => break, // Reached root
        }
    }
}

// Create minimal jj configuration required for loading repos
fn create_minimal_jj_config() -> Option<jj_lib::config::StackedConfig> {
    use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
    use std::env;

    let mut config = StackedConfig::empty();

    // Add required defaults (based on jj-lib's misc.toml)
    let defaults_toml = r#"
        [fsmonitor]
        backend = "none"
        [git]
        abandon-unreachable-commits = true
        auto-local-bookmark = false
        executable-path = "git"
        write-change-id-header = true
        colocate = true
        [merge]
        hunk-level = "line"
        same-change = "accept"
        [operation]
        hostname = "localhost"
        username = "user"
        [signing]
        backend = "none"
        behavior = "keep"
        [signing.backends.gpg]
        allow-expired-keys = false
        program = "gpg"
        [signing.backends.gpgsm]
        allow-expired-keys = false
        program = "gpgsm"
        [signing.backends.ssh]
        program = "ssh-keygen"
        [ui]
        conflict-marker-style = "diff"
        [user]
        name = "path-git-format"
        email = "path-git-format@localhost"
        [working-copy]
        eol-conversion = "none"
    "#;

    if let Ok(layer) = ConfigLayer::parse(ConfigSource::Default, defaults_toml) {
        config.add_layer(layer);
    }

    // Try to load user config if available (will override defaults)
    if let Ok(home) = env::var("HOME") {
        let config_path = std::path::PathBuf::from(home).join(".config/jj/config.toml");
        if config_path.exists() {
            if let Ok(layer) = ConfigLayer::load_from_file(ConfigSource::User, config_path) {
                config.add_layer(layer);
            }
        }
    }

    Some(config)
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
