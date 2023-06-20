mod database;
use clap::Parser;
use database::*;
use lazy_static::lazy_static;
use regex::RegexSet;
use std::{eprintln, fs::read_dir, io, path::PathBuf, println, process::exit};

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// Path of video folder(s)
    #[arg(short, long, value_parser = valid_paths, num_args = 1.. )]
    path: Vec<PathBuf>,

    /// Output Folder for Plex formatted media
    #[arg(short,long,value_parser = valid_paths, num_args = 1) ]
    output_path: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = setup_database(URL).await?;
    let args = Cli::parse();
    for path in &args.path {
        let mut name = std::ffi::OsStr::new("");
        loop {
            println!(
                "What would you like the entries for {} to be tittled?: ",
                path.to_str().unwrap()
            );
            let mut ans = String::new();
            io::stdin().read_line(&mut ans)?;
            ans = String::from(ans.trim_end());
            if !valid_name(&ans) || show_in_database(&db, &ans).await? {
                continue;
            } else {
                name = std::ffi::OsStr::new(&ans);
                insert_show(&db, name.to_str().unwrap()).await?;
                break;
            }
        }
        print_directory(path.to_path_buf())?;
    }
    Ok(())
}

fn valid_paths(s: &str) -> anyhow::Result<PathBuf> {
    let path: PathBuf = s.parse()?;
    if !path.is_dir() {
        eprintln!("\"{}\" is not a directory", path.to_str().unwrap());
        exit(1);
    }
    Ok(path)
}

fn print_directory(path: PathBuf) -> anyhow::Result<()> {
    let mut files = read_dir(path)?
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|x| x.is_ok())
        .flatten()
        .collect::<Vec<_>>()
        .into_iter()
        .map(|x| x.file_name().to_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    files.sort_by(|a, b| natord::compare(&a.to_ascii_lowercase(), &b.to_ascii_lowercase()));
    files.into_iter().for_each(|f| println!("{f}"));

    Ok(())
}

fn valid_name(name: &str) -> bool {
    lazy_static! {
        static ref REGEXES: RegexSet = RegexSet::new(&[
            r#"[<>:"/\|?*\\]"#,
            r#"COM[0-9]"#,
            r#"LPT[0-9]"#,
            r#"NUL"#,
            r#"PRN"#,
            r#"AUX"#
        ])
        .unwrap();
    }
    if name.len() == 0
        || REGEXES.is_match(name)
        || name.contains("\0")
        || name.chars().nth(name.len() - 2).unwrap() == '.'
        || name.chars().nth(name.len() - 2).unwrap() == ' '
    {
        false
    } else {
        true
    }
}
