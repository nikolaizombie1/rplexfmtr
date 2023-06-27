use std::path::PathBuf;
use tabled::Tabled;

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};

/// URL for sqlite database.
///
/// Uses a transient in memery database to track the files in an organized fashion.
pub const URL: &str = "sqlite::memory:";

/// Strcut to hold a show name from a sqlx query.
///
/// This is used in conjunction with the sqlx::query_as function to get the series names from the database.
///
/// # Panics
///
/// The sqlx::query_as function will panic if the selected collumns from the table do not match the type and ammount and name of fields being seleted by the query.
///
/// # Examples
///
/// ```
/// # fn main() -> anyhow::Result<()> {
///     # let db = setup_database().await;
///     # insert_episode(&db,"Show",1,1,"/","/etc");
///
///     let results = sqlx::query_as::<_, Show>("SELECT DISTINCT series_name FROM episodes;")
///         .fetch_all(db)
///         .await;
/// }
/// ```
#[derive(Clone, FromRow, Debug)]
pub struct Show {
    /// Holds the series_name for the sqlx::query_as for select_all_shows()
    pub series_name: String,
}

#[derive(Clone, FromRow, Debug, Tabled)]
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
