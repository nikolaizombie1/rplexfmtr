use crate::*;
use clap::Parser;
use colored::*;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Path of video folder(s)
    #[arg(short, long, value_parser = valid_paths, num_args = 1.. )]
    pub path: Vec<PathBuf>,

    /// Output Folder for Plex formatted media
    #[arg(short,long,value_parser = valid_paths, num_args = 1) ]
    pub output_path: PathBuf,
}

pub async fn move_files(db: &sqlx::SqlitePool, args: &Cli) -> anyhow::Result<()> {
    for show in select_all_shows(&db).await? {
        for episode in select_all_episodes(&db, &show.series_name)
            .await?
            .into_iter()
        {
            std::fs::create_dir_all(args.output_path.join(episode.clone().series_name).join(
                String::from("Season ".to_owned() + &episode.season.to_string()),
            ))?;
            std::fs::copy(episode.clone().old_path, episode.clone().new_path)?;
            std::fs::remove_file(episode.clone().old_path)?;
        }
    }
    Ok(())
}

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

pub fn print_directory(path: PathBuf) -> anyhow::Result<()> {
    let mut files = get_file_names(&get_files(path).unwrap())?;
    files.sort_by(|a, b| natord::compare(&a.to_ascii_lowercase(), &b.to_ascii_lowercase()));
    for (num, file) in files.into_iter().enumerate() {
        println!("{num}. {file}");
    }
    Ok(())
}

pub fn get_file_names(files: &Vec<DirEntry>) -> anyhow::Result<Vec<String>> {
    Ok(files
        .into_iter()
        .map(|x| x.file_name().to_str().unwrap().to_owned())
        .collect::<Vec<_>>())
}

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
            .to_string()
    );
    Ok(())
}
