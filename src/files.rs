use crate::*;
use clap::Parser;
use colored::*;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::path::PathBuf;

/// Struct for the command line argument parser that allows for multiple input paths (minimum of 1) and a single output path.
///
/// This struct uses the [`clap`] crate syntax to have two flags:
/// 1. path: Input paths that contain the media files to be renamed which are verified to be correctly inputted by [`valid_paths()`]. One or more paths can be inputed with a single use of a of the -p flag or each path can be specified by an individual -p flag.
/// 2. output_path: Output path for the Plex® Media Server formatted media which is verified by [`valid_paths()`]. A single output path can be specified with the -o flag.
///
/// # Panics
///
/// Should not panic under normal circumstances.
///
/// # Exits
///
/// When [`Cli::parse()`] is called, [`valid_paths()`] will exit with a status code of `1` if an invalid path is given as an argument.
///
/// # Example
/// ```
/// let args = Cli::parse();
/// println!("{}", args.output_path);
/// ```
#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Input path(s) of video folder(s)
    #[arg(short, long, value_parser = valid_paths, num_args = 1.. )]
    pub input_paths: Vec<PathBuf>,

    /// Output Folder for Plex formatted media
    #[arg(short,long,value_parser = valid_paths, num_args = 1) ]
    pub output_path: PathBuf,
}

/// Will move all files from a database to the output_path given by the command line.
///
/// This function will first collect all of the episode entries from the database, create the folder structure for the particular show and season and move the episode to the new_path directory of the given episode. Will run a filesystem rename if the old_path and new_path directories are in the same file system, else will copy the file to new_path and delete file at old_path.
///
/// # Panics
/// - If the file in the old_path of the episode entry no longer exists the method will panic.
/// - If the file in the old_path of the episode entry no longer has permissions to read the file, the method will panic.
/// - If the new_path directory no longer has write permissions, this method will panic.
pub async fn move_files(db: &sqlx::SqlitePool, args: &Cli) -> anyhow::Result<()> {
    for show in select_all_shows(db).await? {
        for episode in select_all_episodes(db, &show.series_name)
            .await?
            .into_iter()
        {
            std::fs::create_dir_all(args.output_path.join(episode.clone().series_name).join(
                "Season ".to_owned() + &episode.season.to_string(),
            ))?;
            match std::fs::rename(episode.clone().old_path, episode.clone().new_path) {
                Ok(_) => {}
                Err(_) => {
                    std::fs::copy(episode.clone().old_path, episode.clone().new_path)?;
                    std::fs::remove_file(episode.clone().old_path)?;
                }
            }
        }
    }
    Ok(())
}

/// Given a valid path, will return a [`Result<Vec<std::fs::DirEntry>>`] that are naturally sorted.
///
/// This function first collects the [`Result<std::fs::DirEntry>`] into a vector, later filters that vector so that it now only contains Ok [`std::fs::DirEntry`].
/// Then flattens the Ok entries into a [`Vec<std::fs::DirEntry>`], but this vector may contain folders, which isn't valid.
/// So this vector is filtered again to only contain [`std::fs::DirEntry`] entries that are files and the resulting iterator is collected again to a [`Vec<std::fs::DirEntry>`].
/// Finally the vector containing the valid files is sorted by the file name in a natural order.
///
/// # Panics
/// If the given path does not have read permissions.
///
/// # Example
/// ```
/// let files = get_files("/home/user");
/// ```
pub fn get_files(path: PathBuf) -> anyhow::Result<Vec<DirEntry>> {
    let mut files = read_dir(path)?
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|x| x.is_ok())
        .flatten()
        .collect::<Vec<_>>()
        .into_iter()
        .filter(|x| x.file_type().unwrap().is_file())
        .collect::<Vec<_>>();
    files.sort_by(|a, b| {
        natord::compare(
            a.file_name().to_ascii_lowercase().to_str().unwrap(),
            b.file_name().to_ascii_lowercase().to_str().unwrap(),
        )
    });
    Ok(files)
}

/// Will print out the files in a directory given a valid directory.
///
/// This function gets a vector of file names [`Vec<String>`], from  [`get_file_names()`] which itself gets the files from [`get_files()`] and then will print the entry numbers and file names to standard output.
///
/// # Panics
/// - path argument is invalid.
/// - path does not have read permissions.
/// - The file names cannot be unwraped to a [`&str`].
///
/// # Example
/// ```
/// print_directory("/home/user/");
/// ```
pub fn print_directory(path: PathBuf) -> anyhow::Result<()> {
    let files = get_file_names(&get_files(path).unwrap())?;
    for (num, file) in files.into_iter().enumerate() {
        println!("{num}. {file}");
    }
    Ok(())
}

/// Given a reference to a [`Vec<std::fs::DirEntry>`] that only contains files, will then return a [`Result<Vec<String>>`].
///
/// This function first converts the given vector to an iterator to which  maps the DirEntry to a file name that is a owned string and then collects it into a Vector of [`String`] if the file name can be unwraped to an [`&str`].
///
/// # Panics
/// If the file name cannot be successfully unwrapped to a [`&str`].
///
pub fn get_file_names(files: &[DirEntry]) -> anyhow::Result<Vec<String>> {
    Ok(files
        .iter()
        .map(|x| x.file_name().to_str().unwrap().to_owned())
        .collect::<Vec<_>>())
}

/// Prints the renaming changes before and after in a table to standard output given a database with episode entries.
///
/// The table is of a rounded style with a green border, with individually colored columns as follows:
/// - series_name = Red.
/// - season = Yellow.
/// - episode = Cyan.
/// - old_path = Blue.
/// - new_path = Green.
///
/// First the episodes are all retrieved from the database and pushes the episode entries to a vector.
/// Then the vector is turned into table using [`tabled::Table::new()`] function with the style mentioned above.
pub async fn preview_changes(db: &sqlx::SqlitePool) -> anyhow::Result<()> {
    clearscreen::clear()?;
    let mut entries: Vec<Episode> = Vec::new();
    for show in select_all_shows(db).await? {
        for episode in select_all_episodes(db, &show.series_name).await? {
            entries.push(episode);
        }
    }
    println!(
        "{}",
        tabled::Table::new(entries)
            .with(tabled::settings::Style::rounded())
            .with(
                tabled::settings::style::BorderColor::default()
                    .top(tabled::settings::Color::FG_GREEN)
                    .bottom(tabled::settings::Color::FG_GREEN)
                    .left(tabled::settings::Color::FG_GREEN)
                    .right(tabled::settings::Color::FG_GREEN)
                    .corner_top_left(tabled::settings::Color::FG_GREEN)
                    .corner_top_right(tabled::settings::Color::FG_GREEN)
                    .corner_bottom_left(tabled::settings::Color::FG_GREEN)
                    .corner_bottom_right(tabled::settings::Color::FG_GREEN)
            )
            .with(
                tabled::settings::Modify::new(tabled::settings::object::Columns::single(0)).with(
                    tabled::settings::Format::content(|s| s.bright_red().to_string())
                )
            )
            .with(
                tabled::settings::Modify::new(tabled::settings::object::Columns::single(1)).with(
                    tabled::settings::Format::content(|s| s.yellow().to_string())
                )
            )
            .with(
                tabled::settings::Modify::new(tabled::settings::object::Columns::single(2))
                    .with(tabled::settings::Format::content(|s| s.cyan().to_string()))
            )
            .with(
                tabled::settings::Modify::new(tabled::settings::object::Columns::single(3)).with(
                    tabled::settings::Format::content(|s| s.bright_blue().to_string())
                )
            )
            .with(
                tabled::settings::Modify::new(tabled::settings::object::Columns::single(4)).with(
                    tabled::settings::Format::content(|s| s.bright_green().to_string())
                )
            )
    );
    Ok(())
}
