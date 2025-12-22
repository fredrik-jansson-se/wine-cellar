use std::str::FromStr;

use anyhow::Context;

#[derive(sqlx::FromRow, Debug)]
pub(crate) struct Wine {
    pub id: i64,
    pub name: String,
    pub year: i64,
    pub image_b64: Option<String>,
    pub image_thumbnail_b64: Option<String>,
}

#[derive(sqlx::FromRow, Debug)]
pub(crate) struct WineInvEvent {
    pub dt: chrono::NaiveDateTime,
    pub bottles: i64,
}

pub(crate) async fn connect() -> anyhow::Result<sqlx::SqlitePool> {
    let cfg = sqlx::sqlite::SqliteConnectOptions::from_str(
        &std::env::var("DATABASE_URL").context("DATABASE_URL not set")?,
    )?
    .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Full)
    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
    .create_if_missing(true);
    let db = sqlx::SqlitePool::connect_with(cfg).await?;
    Ok(db)
}

pub(crate) async fn wines(db: &sqlx::SqlitePool) -> anyhow::Result<Vec<Wine>> {
    let res = sqlx::query_as!(Wine, "SELECT * from wines")
        .fetch_all(db)
        .await?;
    Ok(res)
}

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

pub(crate) async fn add_wine(db: &sqlx::SqlitePool, name: &str, year: i64) -> anyhow::Result<()> {
    sqlx::query!("INSERT INTO wines (name, year) VALUES ($1, $2)", name, year)
        .execute(db)
        .await?;
    Ok(())
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
    sqlx::query!("DELETE FROM wines WHERE id=$1", wine_id)
        .execute(&mut *trans)
        .await?;
    trans.commit().await?;
    Ok(())
}

pub(crate) async fn get_wine(db: &sqlx::SqlitePool, id: i64) -> anyhow::Result<Wine> {
    let res = sqlx::query_as!(Wine, "SELECT * from wines WHERE id=$1", id)
        .fetch_one(db)
        .await?;
    Ok(res)
}

pub(crate) async fn get_wine_grapes(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Vec<String>> {
    let res = sqlx::query_scalar!(
        "SELECT grape_name FROM wine_grapes WHERE wine_id=$1",
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

pub(crate) async fn get_grapes(db: &sqlx::SqlitePool) -> anyhow::Result<Vec<String>> {
    let res = sqlx::query_scalar!("SELECT name FROM grapes")
        .fetch_all(db)
        .await?;
    Ok(res)
}

// Assumes b64 encoded image and thumbnail
pub(crate) async fn set_wine_image(
    db: &sqlx::SqlitePool,
    wine_id: i64,
    image: &str,
    thumbnail: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE wines SET image_b64=$2, image_thumbnail_b64=$3 WHERE id=$1",
        wine_id,
        image,
        thumbnail
    )
    .execute(db)
    .await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
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
