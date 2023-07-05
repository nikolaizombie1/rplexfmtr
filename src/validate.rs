use once_cell::unsync::Lazy;
use regex::{Regex, RegexSet};
use std::path::PathBuf;
use std::process::exit;

/// Verifies if a given string can be parsed as a Path and is a directory. Exits if the string is not a directory. Returns the inputted string if it is a valid directory.
///
/// Function will exit with an exit code of `1` if a given string is either not a directory.
///
/// # Panic
/// - The input string cannot be parsed to a path.
///
/// # Example
/// ```
/// valid_paths("/home/user/");
/// ```
pub fn valid_paths(s: &str) -> anyhow::Result<PathBuf> {
    let path: PathBuf = s.parse()?;
    if !path.is_dir() {
        eprintln!("\"{}\" is not a directory", path.to_str().unwrap());
        exit(1);
    }
    Ok(path)
}

/// Verifies if a given string does not contain neither the substring(s):
/// - <
/// - \>
/// - :
/// - /
/// - \\
/// - |
/// - ?
/// - \*
/// - COM0, COM1 ... COM9
/// - LPT0, LPT1 ... LPT9
/// - NUL
/// - PRN
/// - AUX
/// If the given string does not contain any of the aforementioned substrings, the function returns true, else returns false.
///
/// This function statically loaded and compiled regular expressions from the [`regex`] crate using the [`once_cell::unsync::Lazy::new()`] function. The regular expressions are compiled only when the function is called and only compile once.
///
/// # Panics
/// Under normal circumstances the function should not panic but if the regular expressions are modified, it can panic due to either the regular expressions failing to compile or a parse of a string to a usize fails due to a change in the regular expressions.
///
/// # Example
/// ```
/// let good_name = valid_name("Show");
/// assert_eq!(good_name,true);
/// ```
pub fn valid_name(name: &str) -> bool {
    let regexes: Lazy<RegexSet> = Lazy::new(|| {
        RegexSet::new([
            r#"[<>:"/\|?*\\]"#,
            r#"COM[0-9]"#,
            r#"LPT[0-9]"#,
            r#"NUL"#,
            r#"PRN"#,
            r#"AUX"#,
        ])
        .unwrap()
    });
    !(name.is_empty()
        || regexes.is_match(name)
        || name.contains('\0')
        || name.chars().nth(name.len() - 2).unwrap() == '.'
        || name.chars().nth(name.len() - 2).unwrap() == ' ')
}

/// Given a string and the amount of files in a folder, will return a [`Result<Vec<usize>>`] containing the indexes of the selected files in either:
/// - A dual ended range. eg.(0-3)
/// - A left ended range. eg.(0-)
/// - A right ended range. eg.(-5)
/// - Comma separated values. eg.(1,2,3)
/// - Single Values. eg.(1 2 3)
///
/// The input string, if it has multiple ranges, need to be space separated. The amount_files needs to be the length of the vector of the files of which the user has selected.
///
/// # Panics
/// Under normal circumstances the function should not panic but if the regular expressions are modified, it can panic due to either the regular expressions failing to compile or a parse of a string to a usize fails due to a change in the regular expressions.
///
/// # Example
/// ```
/// let result = parse_range(6,"0 1-2 3,4 5")
/// assert_eq!(result, vec![0,1,2,3,4,5]);
/// ```
pub fn parse_range(amount_files: usize, range: String) -> anyhow::Result<Vec<usize>> {
    let mut file_numbers: Vec<usize> = Vec::new();
    let dualendedrange: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\d+-\d+$"#).unwrap());
    let leftendedrange: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\d+-$"#).unwrap());
    let rightendedrange: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^+-\d$"#).unwrap());
    let csv: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d+,)+\d$"#).unwrap());
    let single: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\d$"#).unwrap());
    let ranges = range
        .split_ascii_whitespace()
        .map(|x| x.to_owned())
        .collect::<Vec<_>>();
    if range.is_empty() {
        for num in 0..amount_files {
            file_numbers.push(num);
        }
    } else {
        for r in ranges {
            if dualendedrange.is_match(&r) {
                let nums = r.split('-').collect::<Vec<&str>>();
                let left: usize = nums.first().unwrap().parse()?;
                let right: usize = nums.get(1).unwrap().parse()?;
                if left < amount_files && right < amount_files && left <= right {
                    for num in left..(right + 1) {
                        file_numbers.push(num);
                    }
                }
            } else if leftendedrange.is_match(&r) {
                let nums = r.split('-').collect::<Vec<&str>>();
                let left: usize = nums.first().unwrap().parse()?;
                if left < amount_files {
                    for num in left..amount_files {
                        file_numbers.push(num);
                    }
                }
            } else if rightendedrange.is_match(&r) {
                let nums = r.split('-').collect::<Vec<&str>>();
                let right: usize = nums.get(1).unwrap().parse()?;
                if right < amount_files {
                    for num in 0..(right + 1) {
                        file_numbers.push(num);
                    }
                }
            } else if csv.is_match(&r) {
                let nums = r
                    .split(',')
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|x| x.parse().unwrap())
                    .collect::<Vec<usize>>();
                for num in nums {
                    if num < amount_files {
                        file_numbers.push(num);
                    }
                }
            } else if single.is_match(&r) {
                let num: usize = r.parse().unwrap();
                if num < amount_files {
                    file_numbers.push(num);
                }
            }
        }
    }
    Ok(file_numbers)
}
