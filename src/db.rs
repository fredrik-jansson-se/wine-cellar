#[derive(sqlx::FromRow, Debug)]
pub(crate) struct Wine {
    pub id: i64,
    pub name: String,
    pub year: i64,
    pub image: Option<Vec<u8>>,
}

#[derive(sqlx::FromRow, Debug)]
pub(crate) struct WineInvEvent {
    pub wine_id: i64,
    pub dt: chrono::NaiveDateTime,
    pub bottles: i64,
}

pub(crate) async fn connect() -> anyhow::Result<sqlx::SqlitePool> {
    let db = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
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
        "SELECT * from wine_inventory_events WHERE wine_id=$1",
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
