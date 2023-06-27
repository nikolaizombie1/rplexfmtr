use std::path::PathBuf;
use tabled::Tabled;

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};

/// URL for sqlite database.
///
/// Uses a transient in memory database to track the episode entries.
pub const URL: &str = "sqlite::memory:";

/// Struct to hold a show name from a sqlx query.
///
/// This is used in conjunction with the [`sqlx::query_as()`] function to get the series names from the database.
///
/// # Panics
///
/// The sqlx::query_as function will panic if the selected columns from the table do not match the type and ammount and name of fields being seleted by the query.
///
/// # Examples
/// ```
///     let results = sqlx::query_as::<_, Show>("SELECT DISTINCT series_name FROM episodes;")
///         .fetch_all(db);
/// ```
#[derive(Clone, FromRow, Debug)]
pub struct Show {
    /// Holds the series_name for [`select_all_shows()`]
    pub series_name: String,
}

/// Struct to hold an episode entry from the database.
///
/// This is used in conjunction with the [`sqlx::query_as()`] function to get the episode entries from a database query.
///
/// # Panics
///
/// The sqlx::query_as function will panic if the selected collumns from the table do not match the type and ammount and name of fields being seleted by the query.
///
/// # Examples
/// ```
///     # let db = rplexfmtr::setup_database().await;
///     # rplexfmtr::insert_episode(&db,"Show",1,1,"/home/user/show1.mkv","/home/user/output/show S1E1.mkv");
///     sqlx::query_as::<_, Episode>(
///     "SELECT DISTINCT series_name, season, episode, old_path, new_path FROM episodes WHERE series_name = ? ORDER BY LENGTH(series_name), series_name, LENGTH(season), season, LENGTH(old_path), old_path;");
/// ```
#[derive(Clone, FromRow, Debug, Tabled)]
pub struct Episode {
    /// Holds the series name for [`select_all_episodes()`].
    pub series_name: String,
    /// Holds the season number for [`select_all_episodes()`].
    pub season: u32,
    /// Holds the episode number for [`select_all_episodes()`].
    episode: u32,
    /// Holds the current path for the file for the current episode for [`select_all_episodes()`].
    pub old_path: String,
    /// Holds the output path for the file for the current episode for [`select_all_episodes()`].
    pub new_path: String,
}

/// Setup the database connection and tables and returns the database connection.
///
/// Should be used before any database operation is performed since it returns the executor for the in memory database.
///
/// # Panics
///
/// Will Panic if the URL is invalid or if the function is called more than once.
///
/// # Examples
///
/// ```
/// # use crate::verify::*;
/// let db = setup_database(URL);
/// ```
pub async fn setup_database(url: &str) -> anyhow::Result<sqlx::Pool<Sqlite>> {
    Sqlite::create_database(url).await?;

    let db = SqlitePool::connect(url).await?;

    sqlx::query("CREATE TABLE episodes (series_name TEXT, season INTEGER NOT NULL, episode INTEGER NOT NULL, old_path TEXT NOT NULL UNIQUE, new_path TEXT NOT NULL UNIQUE);")
        .execute(&db)
        .await?;

    Ok(db)
}

///  Given a database connection will return all distinct series_name values from the database.
///
///  This function will return a [`Vec<Show>`], if the database is empty the return vector will also be empty.
///
///  # Panics
///
///  Panics if given database connection does not contain the episode table created in [`setup_database()`].
///
///  # Examples
///  ```
///  # let db = setup_database();
///  let result = select_all_shows(&db);
///  ```
///
pub async fn select_all_shows(db: &SqlitePool) -> anyhow::Result<Vec<Show>> {
    Ok(
        sqlx::query_as::<_, Show>("SELECT DISTINCT series_name FROM episodes;")
            .fetch_all(db)
            .await?,
    )
}

/// Will insert an episode entry into the database given a:
/// 1. database connection
/// 2. series name
/// 3. season number
/// 4. episode number
/// 5. current file path
/// 6. output file path
///
/// This will insert the given episode into the episodes table of the database.
///
/// # Panics
/// Will panic with a series name, season number and episode number for a given entry is already in the database since the old_path and new_path columns are unique.
///
/// # Examples
/// ```
/// insert_episode(&db,"Show",1,1,"/home/user/show1.mkv","/home/user/output/show S1E1.mkv");
/// ```
/// **NOTE:** The series_name should be first verified by [`crate::validate::valid_name()`] to ensure that old_path and new_path are valid.
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

///  Given a database connection will return all distinct series_name values from the database given a database connection and a series name.
///
///  This function will return a [`Vec<Episode>`], if the database is empty the return vector will also be empty. The entries are sorted in a natural order.
///
///  # Panics
///
///  Panics if given database connection does not contain the episode table created in [`setup_database()`].
///
///  # Examples
///  ```
///  # let db = setup_database();
///  let result = select_all_episodes(&db,"Show");
///  ```

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
