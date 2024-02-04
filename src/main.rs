use clap::Parser;
use std::io;
use std::path::Path;

// iterate the stdin lines: https://doc.rust-lang.org/std/io/struct.Stdin.html#method.lines

struct Options {
    format: Option<String>,
    cwd: Option<Box<Path>>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // the format in each line.
    #[arg(short, long)]
    format: Option<String>,
    #[arg(short, long)]
    cwd: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let options = Options {
        format: args.format,
        cwd: None,
        // cwd: Box::new(Path::from(args.cwd.
    };

    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer)?;

    println!("{}", buffer);

    Ok(())
}
