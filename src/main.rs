#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

//! Quick and easy batch file renaming for Plex® Media Server
//!
//! A fast an easy command line utility for renaming files for Plex® Media server to recognize.
//! This utility only works for for TV Shows
//!
//! # Usage
//! plexfmtr -p \[INPUT_FOLDER\] -o \[OUTPUT_FOLDER\]

/// Holds the all sqlite database related functions and structs
pub mod database;
pub mod files;
pub mod validate;
use clap::Parser;
use validate::*;
use colored::*;
use database::*;
use files::*;
use std::{io, println, process::exit};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = setup_database(URL).await?;
    let args = Cli::parse();
    for path in &args.path {
        let name: String;
        loop {
            println!(
                "What would you like the entries for {} to be tittled?: ",
                path.to_str().unwrap().green()
            );
            let mut ans = String::new();
            io::stdin().read_line(&mut ans)?;
            ans = String::from(ans.trim_end());
            if !valid_name(&ans) {
                continue;
            } else {
                name = ans;
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
        selected_files.sort_by(|a, b| {
            natord::compare(
                a.file_name().to_ascii_lowercase().to_str().unwrap(),
                b.file_name().to_ascii_lowercase().to_str().unwrap(),
            )
        });
        let season: u32;
        loop {
            println!("What season do these files belong to?");
            let mut ans: String = String::new();
            io::stdin().read_line(&mut ans)?;
            ans = String::from(ans.trim_end());
            break match ans.parse::<u32>() {
                Ok(x) => {
                    season = x;
                }
                Err(_) => {
                    continue;
                }
            };
        }

        for (index, file) in selected_files.into_iter().enumerate() {
            insert_episode(
                &db,
                &name,
                season,
                (index as u32) + 1,
                std::fs::canonicalize(file.path())?,
                args.output_path
                    .to_owned()
                    .join(&name)
                    .join(String::from("Season ".to_owned() + &season.to_string()))
                    .join(String::from(
                        name.clone()
                            + " S"
                            + &season.to_string()
                            + "E"
                            + &(index + 1).to_string()
                            + "."
                            + file
                                .file_name()
                                .to_str()
                                .unwrap()
                                .split(".")
                                .collect::<Vec<_>>()
                                .last()
                                .unwrap(),
                    )),
            )
            .await?;
        }
        clearscreen::clear()?;
    }
    println!("Would you like to preview the changes [y/n]:");
    let mut ans: String = String::new();
    io::stdin().read_line(&mut ans)?;
    ans = String::from(ans.trim_end());
    match ans.to_lowercase() == "y" {
        true => {
            preview_changes(&db).await?;
        }
        false => {}
    }
    println!("Would you like to execute these changes [y/n]:");
    let mut ans: String = String::new();
    io::stdin().read_line(&mut ans)?;
    ans = String::from(ans.trim_end());
    match ans.to_ascii_lowercase() == "y" {
        true => move_files(&db, &args).await?,
        false => exit(0),
    }

    println!(
        "Files renamed succesfully, Located at {}.",
        args.output_path.to_str().unwrap().green()
    );
    Ok(())
}
