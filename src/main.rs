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
}

#[derive(Debug)]
struct GitInfo<'a> {
    path_index: usize,
    segments: Vec<&'a str>,
    branch: Option<String>,
}

impl<'a> GitInfo<'a> {
    fn new(segments: Vec<&'a str>, path_index: usize) -> Self {
        GitInfo {
            path_index,
            segments,
            branch: None,
        }
    }

    fn path_str(&self) -> Option<&str> {
        self.segments.get(self.path_index).copied()
    }

    // git rev-parse --resolve-git-dir <path> -- git symbolic-ref --short HEAD
    async fn update_branch(&mut self) {
        let path = self.path_str();
        if path.is_none() {
            return;
        }
        let repo = Repository::open(PathBuf::from(path.unwrap())).ok();
        let Some(repo) = repo else {
            return;
        };
        let branch: String = repo.head().map_or("".to_owned(), |head| {
            head.shorthand().map_or("".to_owned(), |x| x.to_owned())
        });
        self.branch = Some(branch);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // the format in each line.
    #[arg(short, long)]
    format: Option<String>,
    // parse input
    #[arg(short, long)]
    nth: Option<usize>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let opts = CliOptions {
        format: args.format,
        nth: args.nth.map_or(0, |x| x),
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
        writter.write_all(result_line.as_bytes()).await?;
        writter.write_all(b"\n").await?;
    }

    writter.shutdown().await?;

    Ok(())
}

async fn process_line(opts: &CliOptions, line: &str) -> Option<String> {
    // trim line .then separate line by space
    let segments = line.trim().split(' ').collect::<Vec<&str>>();
    let mut gitinfo = GitInfo::new(segments, opts.nth);
    gitinfo.update_branch().await;

    if gitinfo.branch.is_none() {
        return Some(gitinfo.path_str().unwrap_or("").to_owned());
    }

    let mut vars = HashMap::<String, &str>::new();
    vars.insert("path".to_owned(), gitinfo.path_str().unwrap_or(""));
    vars.insert("branch".to_owned(), gitinfo.branch.as_deref().unwrap_or(""));

    let fmt = opts.format.as_deref().unwrap_or("{path} {branch}");
    strfmt(fmt, &vars).ok()
}
