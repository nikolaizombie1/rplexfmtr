use lazy_static::lazy_static;
use regex::{Regex, RegexSet};
use std::path::PathBuf;
use std::process::exit;

pub fn valid_paths(s: &str) -> anyhow::Result<PathBuf> {
    let path: PathBuf = s.parse()?;
    if !path.is_dir() {
        eprintln!("\"{}\" is not a directory", path.to_str().unwrap());
        exit(1);
    }
    Ok(path)
}

pub fn valid_name(name: &str) -> bool {
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

pub fn parse_range(ammount_files: usize, range: String) -> anyhow::Result<Vec<usize>> {
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
