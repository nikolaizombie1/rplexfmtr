#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

//! Quick and easy batch file renaming for Plex® Media Server
//!
//! A fast an easy command line utility for renaming files for Plex® Media server to recognize.
//! This utility only works for for TV Shows
//!
//! # Usage
//! plexfmtr -i \[input_folder(s)\] -o \[output_folder\]

/// Holds the all sqlite database related functions and structs
pub mod database;
/// Contains all file system manipulation and display functions and structs as well as command line argument and path parsing.
pub mod files;
/// Contains all functions to validate user input
pub mod validate;
use clap::Parser;
use colored::*;
use database::*;
use files::*;
use std::{io, println, process::exit};
use validate::*;

/// The main function for rplexfmtr.\
///
/// First, the main function  initialized the transient, in memory, database using [`database::setup_database()`].
/// Then parses and verifies command line arguments using [`clap`] and [`validate::valid_paths()`].\
///
/// Then iterates through the input paths and prompts the user for what series name would they like the files to correspond to.
/// If the user inputs an invalid name, it will prompt the user again to enter a name until a valid name is given.\
///
/// Once a valid name is provided, the user is prompted to choose which files they would like to be renamed.
/// Once again, if the user provides an invalid input, they will be prompted to input a season number again unit a valid input is provided.\
///
/// Then, the validated set of files will be inputted to the database.
/// If more than one input directory was provided, the user will be prompted to input the information for that set of files, repeating the process.\
///
/// Once all input directories have been processed, the user will be prompted to preview the changes.
/// If the user inputs 'y', a table containing the series_name, season, episode, current_path and new_path will be displayed to the user.
/// Else, it does not display the changes.\
///
/// Finally, the user will be prompted if they would like to execute the changes.
/// If the user inputs 'y', the renaming process will commence and the files will be renamed and moved to the output directory following the Plex® Media Server folder structure.
/// If all files are renamed successfully, the user will be shown that the files have been moved successfully and show the the location of the renamed files.
#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    let db = setup_database(URL).await?;
    let args = Cli::parse();
    for path in &args.input_paths {
        let name: String;
        loop {
            println!(
                "What would you like the entries for {} to be titled?: ",
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
                file.path(),
                args.output_path
                    .to_owned()
                    .join(&name)
                    .join("Season ".to_owned() + &season.to_string())
                    .join(
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
                                .split('.')
                                .collect::<Vec<_>>()
                                .last()
                                .unwrap(),
                    ),
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
        "Files renamed successfully, Located at {}.",
        args.output_path.to_str().unwrap().green()
    );
    Ok(())
}
