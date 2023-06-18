use sqlx::{migrate::MigrateDatabase, sqlite::SqliteQueryResult, FromRow, Sqlite, SqlitePool};

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
async fn main() -> anyhow::Result<()> {
    let db = setup_database(URL).await?;

    Ok(())
}

async fn setup_database(url: &str) -> anyhow::Result<sqlx::Pool<Sqlite>> {
    Sqlite::create_database(url).await?;

    let db = SqlitePool::connect(url).await?;

    sqlx::query("CREATE TABLE shows (series_name TEXT PRIMARY KEY NOT NULL);")
        .execute(&db)
        .await?;

    sqlx::query("CREATE TABLE episodes (series_name TEXT, season INTEGER NOT NULL, episode INTEGER NOT NULL, FOREIGN KEY (series_name) REFERENCES shows (series_name) ON DELETE CASCADE ON UPDATE CASCADE);")
        .execute(&db)
        .await?;

    Ok(db)
}

async fn insert_show(db: &SqlitePool, series_name: &str) -> anyhow::Result<SqliteQueryResult> {
    Ok(sqlx::query("INSERT INTO shows(series_name) VALUES (?);")
        .bind(series_name)
        .execute(&mut db.acquire().await?)
        .await?)
}

async fn select_show(db: &SqlitePool, show: &str) -> anyhow::Result<Vec<Show>> {
    Ok(
        sqlx::query_as::<_, Show>("SELECT series_name FROM shows WHERE series_name = ?;")
            .bind(show)
            .fetch_all(&mut db.acquire().await?)
            .await?,
    )
}

async fn select_show_like(db: &SqlitePool, show: &str) -> anyhow::Result<Vec<Show>> {
    Ok(
        sqlx::query_as::<_, Show>("SELECT series_name FROM shows WHERE series_name LIKE ?;")
            .bind(show)
            .fetch_all(&mut db.acquire().await?)
            .await?,
    )
}

async fn insert_episode(
    db: &SqlitePool,
    series_name: &str,
    season: u32,
    episode: u32,
) -> anyhow::Result<SqliteQueryResult> {
    Ok(
        sqlx::query("INSERT INTO episodes (series_name, season, episode) VALUES (?,?,?)")
            .bind(series_name)
            .bind(season)
            .bind(episode)
            .execute(db)
            .await?,
    )
}

async fn select_all_episodes(db: &SqlitePool, series_name: &str) -> anyhow::Result<Vec<Episode>> {
    Ok(sqlx::query_as::<_, Episode>(
        "SELECT series_name, season, episode FROM episodes WHERE series_name = ? ORDER BY episode;",
    )
    .bind(series_name)
    .fetch_all(db)
    .await?)
}

async fn select_all_episodes_from_season(
    db: &SqlitePool,
    series_name: &str,
    season: u32,
) -> anyhow::Result<Vec<Episode>> {
    Ok(sqlx::query_as::<_, Episode>(
        "SELECT series_name, season, episode FROM episodes WHERE series_name = ? AND season = ? ORDER BY season, episode;",
    )
    .bind(series_name)
    .bind(season)
    .fetch_all(db)
    .await?)
}
