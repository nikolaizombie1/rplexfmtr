mod database;
use clap::Parser;
use database::*;
use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use std::{
    eprintln,
    fs::{read_dir, DirEntry},
    io,
    path::PathBuf,
    println,
    process::exit,
};

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
        let mut name = String::new();
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
                name = ans;
                insert_show(&db, &name).await?;
                break;
            }
        }
        let files = get_files(path.to_owned())?;
        print_directory(path.to_path_buf())?;
        println!("Which files would you like to choose?");
        let mut selection: String = String::new();
        io::stdin().read_line(&mut selection)?;
        selection = String::from(selection.trim_end());
        let files_numbers = parse_range(files.len(), selection)?;
        let mut selected_files: Vec<_> = Vec::new();
        for (index, file) in files.into_iter().enumerate() {
            if files_numbers.contains(&index) {
                selected_files.push(file);
            }
        }
        selected_files.sort_by(|a, b| natord::compare(a.file_name().to_ascii_lowercase().to_str().unwrap(), b.file_name().to_ascii_lowercase().to_str().unwrap()) );
        let mut season: usize;
        loop {
            println!("What season do these files belong to?");
            let mut ans: String = String::new();
            io::stdin().read_line(&mut ans)?;
            ans = String::from(ans.trim_end());
            break match ans.parse::<usize>() {
                Ok(x) => {
                    season = x;
                }
                Err(_) => {
                    continue;
                }
            };
        }
        println!("Here is a preview changes of the files.");
        preview_changes(&name, get_file_names(selected_files)?, season)?;
        loop {
            println!("Would you like rename these files? [y/n]:");
            let mut ans: String = String::new();
            io::stdin().read_line(&mut ans)?;
            ans = String::from(ans.trim_end());
            break match ans == "y" {
                true => {}
                false => {
                    exit(0);
                }
            };
        }
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

fn get_files(path: PathBuf) -> anyhow::Result<Vec<DirEntry>> {
    let mut files = read_dir(path)?
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|x| x.is_ok())
        .flatten()
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|x| x.file_type().unwrap().is_file())
        .collect::<Vec<_>>();
    files.sort_by(|a, b| natord::compare(a.file_name().to_ascii_lowercase().to_str().unwrap(), b.file_name().to_ascii_lowercase().to_str().unwrap()) );
    Ok(files)
}

fn print_directory(path: PathBuf) -> anyhow::Result<()> {
    let mut files = get_file_names(get_files(path).unwrap())?;
    files.sort_by(|a, b| natord::compare(&a.to_ascii_lowercase(), &b.to_ascii_lowercase()));
    for (num, file) in files.into_iter().enumerate() {
        println!("{num}. {file}");
    }
    Ok(())
}

fn get_file_names(files: Vec<DirEntry>) -> anyhow::Result<Vec<String>> {
    Ok(files
        .into_iter()
        .map(|x| x.file_name().to_str().unwrap().to_owned())
        .collect::<Vec<_>>())
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

fn parse_range(ammount_files: usize, range: String) -> anyhow::Result<Vec<usize>> {
    let mut file_numbers: Vec<usize> = Vec::new();
    lazy_static! {
        static ref DUALENDEDRANGE: Regex = Regex::new(r#"^\d+-\d+$"#).unwrap();
        static ref LEFTENDEDRANGE: Regex = Regex::new(r#"^\d+-$"#).unwrap();
        static ref RIGHTENDEDRANGE: Regex = Regex::new(r#"^+-\d$"#).unwrap();
        static ref CSV: Regex = Regex::new(r#"^(\d+,)+\d$"#).unwrap();
        static ref SINGLE: Regex = Regex::new(r#"^\d$"#).unwrap();
    }
    let ranges = range
        .split_ascii_whitespace()
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    if range == "" {
        for num in 0..ammount_files {
            file_numbers.push(num);
        }
    } else {
        for r in ranges {
            if DUALENDEDRANGE.is_match(&r) {
                let nums = r.split('-').collect::<Vec<&str>>();
                let left: usize = nums.get(0).unwrap().parse()?;
                let right: usize = nums.get(1).unwrap().parse()?;
                if left < ammount_files && right < ammount_files && left <= right {
                    for num in left..(right + 1) {
                        file_numbers.push(num);
                    }
                }
            } else if LEFTENDEDRANGE.is_match(&r) {
                let nums = r.split('-').collect::<Vec<&str>>();
                let left: usize = nums.get(0).unwrap().parse()?;
                if left < ammount_files {
                    for num in left..ammount_files {
                        file_numbers.push(num);
                    }
                }
            } else if RIGHTENDEDRANGE.is_match(&r) {
                let nums = r.split('-').collect::<Vec<&str>>();
                let right: usize = nums.get(1).unwrap().parse()?;
                if right < ammount_files {
                    for num in 0..(right + 1) {
                        file_numbers.push(num);
                    }
                }
            } else if CSV.is_match(&r) {
                let nums = r
                    .split(',')
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|x| x.parse().unwrap())
                    .collect::<Vec<usize>>();
                for num in nums {
                    if num < ammount_files {
                        file_numbers.push(num);
                    }
                }
            } else if SINGLE.is_match(&r) {
                let num: usize = r.parse().unwrap();
                if num < ammount_files {
                    file_numbers.push(num);
                }
            }
        }
    }
    Ok(file_numbers)
}

fn preview_changes(name: &str, files: Vec<String>, season: usize) -> anyhow::Result<()> {
    for (index, file) in files.into_iter().enumerate() {
        let extention = file.split('.').last().unwrap();
        println!("{index}. {file} ----> {name} S{season}E{}.{extention}",(index+1));
    }
    Ok(())
}