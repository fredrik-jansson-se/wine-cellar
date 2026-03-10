use std::str::FromStr;

use anyhow::Context;

pub(crate) struct FoodPairing {
    pub id: i64,
    pub food: String,
}

pub(crate) struct WineWithPairings {
    #[allow(dead_code)]
    pub wine_id: i64,
    pub name: String,
    pub year: i64,
    pub matched_pairings: Vec<String>,
}

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

#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
pub(crate) async fn delete_wine(db: &sqlx::SqlitePool, wine_id: i64) -> anyhow::Result<()> {
    let mut trans = db.begin().await?;
    sqlx::query!("DELETE FROM wine_food_pairings WHERE wine_id=$1", wine_id)
        .execute(&mut *trans)
        .await?;
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
#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
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
#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
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
    let res = sqlx::query_as!(
        WineComment,
        "SELECT comment,dt FROM wine_comments WHERE wine_id=$1 ORDER by dt DESC LIMIT 1",
        wine_id
    )
    .fetch_optional(db)
    .await?;
    Ok(res)
}

#[tracing::instrument(skip(db))]
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

#[tracing::instrument(skip(db))]
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

/// Returns all food pairings for the given wine, ordered by insertion time.
#[tracing::instrument(skip(db))]
pub(crate) async fn get_wine_food_pairings(
    db: &sqlx::SqlitePool,
    wine_id: i64,
) -> anyhow::Result<Vec<FoodPairing>> {
    let res = sqlx::query!(
        "SELECT id, food FROM wine_food_pairings WHERE wine_id = $1 ORDER BY id",
        wine_id
    )
    .fetch_all(db)
    .await?
    .into_iter()
    .map(|r| FoodPairing {
        id: r.id.expect("id is NOT NULL"),
        food: r.food,
    })
    .collect();
    Ok(res)
}

/// Inserts a new food pairing and returns the created record.
/// Returns a DB error (unique constraint) if the pairing already exists
/// case-insensitively on this wine.
#[tracing::instrument(skip(db))]
pub(crate) async fn add_food_pairing(
    db: &sqlx::SqlitePool,
    wine_id: i64,
    food: &str,
) -> anyhow::Result<FoodPairing> {
    let r = sqlx::query!(
        "INSERT INTO wine_food_pairings (wine_id, food) VALUES ($1, $2) RETURNING id, food",
        wine_id,
        food
    )
    .fetch_one(db)
    .await?;
    Ok(FoodPairing {
        id: r.id.expect("id is NOT NULL"),
        food: r.food,
    })
}

/// Deletes a food pairing by its id, scoped to wine_id to prevent cross-wine deletions.
#[tracing::instrument(skip(db))]
pub(crate) async fn remove_food_pairing(
    db: &sqlx::SqlitePool,
    pairing_id: i64,
    wine_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        "DELETE FROM wine_food_pairings WHERE id = $1 AND wine_id = $2",
        pairing_id,
        wine_id
    )
    .execute(db)
    .await?;
    Ok(())
}

/// Searches wines by food pairing using a case-insensitive substring match.
/// Special LIKE characters (`%`, `_`, `\`) in `q` are escaped so they are
/// treated as literals.
#[tracing::instrument(skip(db))]
pub(crate) async fn search_wines_by_food(
    db: &sqlx::SqlitePool,
    q: &str,
) -> anyhow::Result<Vec<WineWithPairings>> {
    let escaped = q
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    let pattern = format!("%{}%", escaped);

    let wines = sqlx::query!(
        r#"SELECT DISTINCT w.wine_id, w.name, w.year
           FROM wines w
           JOIN wine_food_pairings fp ON fp.wine_id = w.wine_id
           WHERE fp.food LIKE $1 ESCAPE '\'
           ORDER BY w.name, w.year"#,
        pattern
    )
    .fetch_all(db)
    .await?;

    let mut result = Vec::new();
    for wine in wines {
        let pairings = sqlx::query_scalar!(
            "SELECT food FROM wine_food_pairings WHERE wine_id = $1 ORDER BY id",
            wine.wine_id
        )
        .fetch_all(db)
        .await?;
        result.push(WineWithPairings {
            wine_id: wine.wine_id.expect("wine_id is NOT NULL"),
            name: wine.name,
            year: wine.year,
            matched_pairings: pairings,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_db() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect to in-memory DB");
        sqlx::migrate!().run(&pool).await.expect("run migrations");
        pool
    }

    #[tokio::test]
    async fn test_add_and_get_food_pairing() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Test Wine", 2020).await.unwrap();

        let pairing = add_food_pairing(&db, wine.wine_id, "grilled salmon")
            .await
            .unwrap();
        assert_eq!(pairing.food, "grilled salmon");

        let pairings = get_wine_food_pairings(&db, wine.wine_id).await.unwrap();
        assert_eq!(pairings.len(), 1);
        assert_eq!(pairings[0].food, "grilled salmon");
    }

    #[tokio::test]
    async fn test_remove_food_pairing() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Test Wine", 2020).await.unwrap();

        let pairing = add_food_pairing(&db, wine.wine_id, "aged cheddar")
            .await
            .unwrap();
        remove_food_pairing(&db, pairing.id, wine.wine_id)
            .await
            .unwrap();

        let pairings = get_wine_food_pairings(&db, wine.wine_id).await.unwrap();
        assert!(pairings.is_empty());
    }

    #[tokio::test]
    async fn test_duplicate_pairing_rejected() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Test Wine", 2020).await.unwrap();

        add_food_pairing(&db, wine.wine_id, "salmon").await.unwrap();
        // Same pairing, different case — should fail
        let err = add_food_pairing(&db, wine.wine_id, "Salmon").await;
        assert!(err.is_err(), "duplicate pairing must be rejected");
    }

    #[tokio::test]
    async fn test_cascade_delete_removes_pairings() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Test Wine", 2020).await.unwrap();
        add_food_pairing(&db, wine.wine_id, "lamb chops")
            .await
            .unwrap();

        delete_wine(&db, wine.wine_id).await.unwrap();

        // The get_wine call should now fail, and pairings should be gone
        let pairings = get_wine_food_pairings(&db, wine.wine_id).await.unwrap();
        assert!(pairings.is_empty());
    }

    #[tokio::test]
    async fn test_search_wines_by_food_match() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Salmon Wine", 2021).await.unwrap();
        add_food_pairing(&db, wine.wine_id, "grilled salmon")
            .await
            .unwrap();

        let results = search_wines_by_food(&db, "salmon").await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].wine_id, wine.wine_id);
    }

    #[tokio::test]
    async fn test_search_wines_by_food_no_match() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Some Wine", 2021).await.unwrap();
        add_food_pairing(&db, wine.wine_id, "grilled salmon")
            .await
            .unwrap();

        let results = search_wines_by_food(&db, "xyz_no_match").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_search_partial_match() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Some Wine", 2021).await.unwrap();
        add_food_pairing(&db, wine.wine_id, "grilled salmon")
            .await
            .unwrap();

        let results = search_wines_by_food(&db, "sal").await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_search_special_chars_treated_as_literal() {
        let db = setup_db().await;
        let wine = add_wine(&db, "Some Wine", 2021).await.unwrap();
        add_food_pairing(&db, wine.wine_id, "steak").await.unwrap();

        // '%' should not match everything — should match nothing since no pairing contains "%"
        let results = search_wines_by_food(&db, "%").await.unwrap();
        assert!(results.is_empty());
    }
}
