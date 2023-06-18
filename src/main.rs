use core::panic;
use std::{eprintln, error::Error, println};

use sqlx::{migrate::MigrateDatabase, FromRow, Sqlite, SqlitePool, sqlite::SqliteQueryResult};

const URL: &str = "sqlite::memory:";

#[derive(Clone, FromRow, Debug)]
struct Show {
    series_name: String,
}

#[derive(Clone, FromRow, Debug)]
struct Episode {
    series_name: String,
    season: u32,
    episode: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    Sqlite::create_database(URL).await?;

    let db = SqlitePool::connect(URL).await?;

    sqlx::query("CREATE TABLE shows (series_name TEXT PRIMARY KEY NOT NULL);")
        .execute(&db)
        .await?;

    sqlx::query("CREATE TABLE episodes (series_name TEXT, season INTEGER NOT NULL, episode INTEGER NOT NULL, FOREIGN KEY (series_name) REFERENCES shows (series_name) ON DELETE CASCADE ON UPDATE CASCADE);")
        .execute(&db)
        .await?;

    println!("{}",insert_show(&db, "UWU").await?);
    println!("{}",insert_show(&db, "gUWU").await?);
    println!("{}",insert_show(&db, "bUWU").await?);
    let shows = select_show_like(&db, "%UWU").await?;
    for show in shows {
        println!("{}",show.series_name);
    }

    Ok(())
}

async fn insert_show(db: &SqlitePool, series_name: &str) -> anyhow::Result<i64> {
    Ok(sqlx::query("INSERT INTO shows(series_name) VALUES (?);").bind(series_name).execute(&mut db.acquire().await.unwrap()).await?.last_insert_rowid())
}

async fn select_show(db: &SqlitePool, show: &str) -> anyhow::Result<Vec<Show>> {
    Ok(sqlx::query_as::<_,Show>("SELECT * FROM shows WHERE series_name = ?;").bind(show).fetch_all(&mut db.acquire().await?).await?)
}

async fn select_show_like(db: &SqlitePool, show: &str) -> anyhow::Result<Vec<Show>> {
    Ok(sqlx::query_as::<_,Show>("SELECT * FROM shows WHERE series_name LIKE ?;").bind(show).fetch_all(&mut db.acquire().await?).await?)
}

async fn insert_episode(db: &SqlitePool, series_name: &str, season: u32, episode: u32) -> Result<(), Box<dyn Error + 'static>> {
    match sqlx::query("INSERT INTO episodes (series_name, season, episode) VALUES (?,?,?)").bind(series_name).bind(season).bind(episode).execute(db).await {
        Ok(_) => {Ok(())},
        Err(error) => {eprintln!("{}", error); Err(Box::new(error))},
    }

}
