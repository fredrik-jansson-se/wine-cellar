use std::str::FromStr;

use anyhow::Context;

#[derive(Debug)]
pub(crate) struct Wine {
    pub wine_id: i64,
    pub name: String,
    pub year: i64,
    pub has_image: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub(crate) struct WineInvEvent {
    pub dt: chrono::NaiveDateTime,
    pub bottles: i64,
}

pub(crate) async fn connect() -> anyhow::Result<sqlx::SqlitePool> {
    let cfg = sqlx::sqlite::SqliteConnectOptions::from_str(
        &std::env::var("DATABASE_URL").context("DATABASE_URL not set")?,
    )
    .context("Open database file")?
    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
    .create_if_missing(true);
    let db = sqlx::SqlitePool::connect_with(cfg)
        .await
        .context("Open database")?;
    Ok(db)
}

pub(crate) async fn wines(db: &sqlx::SqlitePool) -> anyhow::Result<Vec<Wine>> {
    let res = sqlx::query!("SELECT wine_id, name, year, image IS NOT NULL AS has_image from wines")
        .fetch_all(db)
        .await?
        .into_iter()
        // Not sure why we need this conversation here, but not below where we fetch a single
        // wine
        .map(|r| Wine {
            wine_id: r.wine_id, //.wine_id.expect("Will always have id"),
            name: r.name,
            year: r.year,
            has_image: r.has_image != 0,
        })
        .collect();
    Ok(res)
}

pub(crate) async fn get_wine(db: &sqlx::SqlitePool, id: i64) -> anyhow::Result<Wine> {
    let res = sqlx::query!(
        "SELECT wine_id, name, year, image IS NOT NULL AS has_image from wines WHERE wine_id=$1",
        id
    )
    .fetch_one(db)
    .await?;
    Ok(Wine {
        wine_id: res.wine_id,
        name: res.name,
        year: res.year,
        has_image: res.has_image != 0,
    })
}

#[tracing::instrument(skip(db))]
pub(crate) async fn wine_inventory_events(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Vec<WineInvEvent>> {
    let res = sqlx::query_as!(
        WineInvEvent,
        "SELECT dt,bottles from wine_inventory_events WHERE wine_id=$1",
        wine_id
    )
    .fetch_all(db)
    .await?;

    Ok(res)
}

pub(crate) async fn add_wine(db: &sqlx::SqlitePool, name: &str, year: i64) -> anyhow::Result<Wine> {
    let wine_id = sqlx::query_scalar!(
        "INSERT INTO wines (name, year) VALUES ($1, $2) RETURNING wine_id",
        name,
        year
    )
    .fetch_one(db)
    .await?;
    get_wine(db, wine_id).await
}

pub(crate) async fn delete_wine(db: &sqlx::SqlitePool, wine_id: i64) -> anyhow::Result<()> {
    let mut trans = db.begin().await?;
    sqlx::query!("DELETE FROM wine_comments WHERE wine_id=$1", wine_id)
        .execute(&mut *trans)
        .await?;
    sqlx::query!("DELETE FROM wine_grapes WHERE wine_id=$1", wine_id)
        .execute(&mut *trans)
        .await?;
    sqlx::query!(
        "DELETE FROM wine_inventory_events WHERE wine_id=$1",
        wine_id
    )
    .execute(&mut *trans)
    .await?;
    sqlx::query!("DELETE FROM wines WHERE wine_id=$1", wine_id)
        .execute(&mut *trans)
        .await?;
    trans.commit().await?;
    Ok(())
}

pub(crate) struct Grape {
    pub rowid: i64,
    pub name: String,
}
pub(crate) async fn get_grapes(db: &sqlx::SqlitePool) -> anyhow::Result<Vec<Grape>> {
    let res = sqlx::query!("SELECT rowid, name FROM grapes ORDER BY name")
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|r| Grape {
            // This table always has a rowid
            rowid: r.rowid.unwrap(),
            name: r.name,
        })
        .collect();
    Ok(res)
}

pub(crate) async fn get_wine_grapes(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Vec<String>> {
    let res = sqlx::query_scalar!(
        "SELECT grape_name FROM wine_grapes WHERE wine_id=$1 ORDER BY grape_name",
        wine_id
    )
    .fetch_all(db)
    .await?;

    Ok(res)
}
pub(crate) async fn set_wine_grapes(
    db: &sqlx::SqlitePool,
    wine_id: i64,
    grapes: &[&str],
) -> anyhow::Result<()> {
    tracing::info!("set_wine_grapes: {wine_id}: {grapes:?}");
    let mut trans = db.begin().await?;

    sqlx::query!("DELETE FROM wine_grapes WHERE wine_id=$1", wine_id)
        .execute(&mut *trans)
        .await?;

    for grape in grapes {
        sqlx::query!("INSERT INTO wine_grapes VALUES($1, $2)", wine_id, grape)
            .execute(&mut *trans)
            .await?;
    }

    trans.commit().await?;

    Ok(())
}

pub(crate) async fn set_wine_image(
    db: &sqlx::SqlitePool,
    wine_id: i64,
    image: &[u8],
) -> anyhow::Result<()> {
    sqlx::query!("UPDATE wines SET image=$2 WHERE wine_id=$1", wine_id, image,)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct WineComment {
    pub comment: String,
    pub dt: chrono::NaiveDateTime,
}

pub(crate) async fn wine_comments(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Vec<WineComment>> {
    let res = sqlx::query_as!(
        WineComment,
        "SELECT comment,dt FROM wine_comments WHERE wine_id=$1 ORDER by dt DESC",
        wine_id
    )
    .fetch_all(db)
    .await?;
    Ok(res)
}

#[tracing::instrument(skip(db))]
pub(crate) async fn last_wine_comment(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Option<WineComment>> {
    tracing::info!("enter");
    let res = sqlx::query_as!(
        WineComment,
        "SELECT comment,dt FROM wine_comments WHERE wine_id=$1 ORDER by dt DESC LIMIT 1",
        wine_id
    )
    .fetch_optional(db)
    .await?;
    Ok(res)
}

pub(crate) async fn add_wine_comment(
    db: &sqlx::SqlitePool,
    wine_id: i64,
    comment: &str,
    dt: chrono::NaiveDateTime,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO wine_comments (wine_id, comment, dt) VALUES ($1, $2, $3)",
        wine_id,
        comment,
        dt
    )
    .execute(db)
    .await?;
    Ok(())
}

#[tracing::instrument(skip(db))]
pub(crate) async fn add_wine_event(
    db: &sqlx::SqlitePool,
    wine_id: i64,
    bottles: i64,
    dt: chrono::NaiveDateTime,
) -> anyhow::Result<()> {
    tracing::info!("wine event");
    sqlx::query!(
        "INSERT INTO wine_inventory_events (wine_id, bottles, dt) VALUES ($1, $2, $3)",
        wine_id,
        bottles,
        dt
    )
    .execute(db)
    .await?;
    Ok(())
}

pub(crate) async fn wine_image(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Option<Vec<u8>>> {
    let res = sqlx::query_scalar!("SELECT image FROM wines WHERE wine_id=$1", wine_id)
        .fetch_optional(db)
        .await?
        .flatten();
    Ok(res)
}
