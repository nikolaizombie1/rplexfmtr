use std::path::PathBuf;

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};

pub const URL: &str = "sqlite::memory:";

#[derive(Clone, FromRow, Debug)]
pub struct Show {
    pub series_name: String,
}

#[derive(Clone, FromRow, Debug)]
pub struct Episode {
    pub series_name: String,
    pub season: u32,
    episode: u32,
    pub old_path: String,
    pub new_path: String,
}

pub async fn setup_database(url: &str) -> anyhow::Result<sqlx::Pool<Sqlite>> {
    Sqlite::create_database(url).await?;

    let db = SqlitePool::connect(url).await?;

    sqlx::query("CREATE TABLE episodes (series_name TEXT, season INTEGER NOT NULL, episode INTEGER NOT NULL, old_path TEXT NOT NULL UNIQUE, new_path TEXT NOT NULL UNIQUE);")
        .execute(&db)
        .await?;

    Ok(db)
}

pub async fn select_all_shows(db: &SqlitePool) -> anyhow::Result<Vec<Show>> {
    Ok(
        sqlx::query_as::<_, Show>("SELECT DISTINCT series_name FROM episodes;")
            .fetch_all(db)
            .await?,
    )
}

pub async fn insert_episode(
    db: &SqlitePool,
    series_name: &str,
    season: u32,
    episode: u32,
    old_path: PathBuf,
    new_path: PathBuf,
) -> anyhow::Result<SqliteQueryResult> {
    Ok(
        sqlx::query("INSERT INTO episodes (series_name, season, episode, old_path, new_path) VALUES (?,?,?,?,?)")
            .bind(series_name)
            .bind(season)
            .bind(episode)
            .bind(old_path.as_os_str().to_str().unwrap())
            .bind(new_path.as_os_str().to_str().unwrap())
            .execute(db)
            .await?,
    )
}

pub async fn select_all_episodes(
    db: &SqlitePool,
    series_name: &str,
) -> anyhow::Result<Vec<Episode>> {
    Ok(sqlx::query_as::<_, Episode>(
        "SELECT DISTINCT series_name, season, episode, old_path, new_path FROM episodes WHERE series_name = ? ORDER BY LENGTH(series_name), series_name, LENGTH(season), season, LENGTH(old_path), old_path;",
    )
    .bind(series_name)
    .fetch_all(db)
    .await?)
}
