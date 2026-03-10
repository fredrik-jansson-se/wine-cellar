use super::{MDResult, State};
use anyhow::Context;
use image::GenericImageView;

use crate::{db, web::AppError};

// ── Food Pairings ────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, Debug)]
pub(crate) struct AddFoodPairing {
    food: String,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn add_food_pairing(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(form): axum::extract::Form<AddFoodPairing>,
) -> MDResult {
    let food = form.food.trim();
    if food.is_empty() {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Food pairing cannot be empty"
        )));
    }
    if food.len() > 100 {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Food pairing must be 100 characters or less"
        )));
    }
    match db::add_food_pairing(&state.db, wine_id, food).await {
        Ok(_) => {}
        Err(e) => {
            if let Some(sqlx::Error::Database(db_err)) = e.downcast_ref::<sqlx::Error>()
                && db_err.is_unique_violation()
            {
                return Err(AppError::bad_request(anyhow::anyhow!(
                    "This food pairing already exists for this wine"
                )));
            }
            return Err(e.into());
        }
    }
    let pairings = db::get_wine_food_pairings(&state.db, wine_id).await?;
    Ok(super::markup::food_pairings_list_items(&pairings, wine_id))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn remove_food_pairing(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path((wine_id, pairing_id)): axum::extract::Path<(i64, i64)>,
) -> MDResult {
    db::remove_food_pairing(&state.db, pairing_id, wine_id).await?;
    let pairings = db::get_wine_food_pairings(&state.db, wine_id).await?;
    Ok(super::markup::food_pairings_list_items(&pairings, wine_id))
}

// ── Pairings Search ──────────────────────────────────────────────────────────

#[tracing::instrument]
pub(crate) async fn pairings_search() -> MDResult {
    Ok(super::markup::pairings_search_page())
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct PairingsSearchQuery {
    q: Option<String>,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn pairings_search_results(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Query(query): axum::extract::Query<PairingsSearchQuery>,
) -> MDResult {
    let q = query.q.as_deref().unwrap_or("").trim();
    if q.is_empty() {
        return Ok(super::markup::pairings_search_prompt());
    }
    let wines = db::search_wines_by_food(&state.db, q).await?;
    Ok(super::markup::pairings_search_results_markup(&wines, q))
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct AddWine {
    name: String,
    year: i64,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn add_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::Form(wine): axum::Form<AddWine>,
) -> MDResult {
    tracing::info!("add_wine");
    let wine = db::add_wine(&state.db, &wine.name, wine.year).await?;
    tracing::info!("Added: {wine:?}");
    super::markup::wine_table_row(&state, wine, None).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn delete_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    db::delete_wine(&state.db, wine_id).await?;
    super::markup::wine_table_populated(&state).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn post_wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(form): axum::extract::Form<std::collections::HashMap<String, String>>,
) -> MDResult {
    let grapes: Vec<_> = form.values().map(|v| v.as_ref()).collect();
    db::set_wine_grapes(&state.db, wine_id, &grapes).await?;
    super::markup::wine_table_populated(&state).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn get_note(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    let wine = db::get_wine(&state.db, wine_id).await?;
    Ok(super::markup::note_read_view(&wine))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn get_note_edit(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    let wine = db::get_wine(&state.db, wine_id).await?;
    Ok(super::markup::note_edit_form(
        wine_id,
        wine.comment.as_deref().unwrap_or(""),
        None,
    ))
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct SaveComment {
    comment: String,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn save_comment(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(form): axum::extract::Form<SaveComment>,
) -> MDResult {
    let text = form.comment.trim();
    let (comment_value, dt) = if text.is_empty() {
        (None, None)
    } else {
        (Some(text), Some(chrono::Local::now().naive_local()))
    };
    match db::set_wine_comment(&state.db, wine_id, comment_value, dt).await {
        Ok(()) => {
            let wine = db::get_wine(&state.db, wine_id).await?;
            Ok(super::markup::note_read_view(&wine))
        }
        Err(e) => Ok(super::markup::note_edit_form(
            wine_id,
            text,
            Some(&e.to_string()),
        )),
    }
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct BuyWine {
    dt: String,
    bottles: i64,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn buy_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(event): axum::extract::Form<BuyWine>,
) -> MDResult {
    tracing::info!("buy wine");
    let date = chrono::NaiveDate::parse_from_str(&event.dt, "%Y-%m-%d")?;
    let dt = chrono::NaiveDateTime::new(date, chrono::Local::now().naive_local().time());
    db::add_wine_event(&state.db, wine_id, event.bottles, dt).await?;
    super::markup::wine_table_populated(&state).await
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct ConsumeWine {
    dt: String,
    bottles: i64,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn consume_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(event): axum::extract::Form<ConsumeWine>,
) -> MDResult {
    tracing::info!("consume wine");
    let date = chrono::NaiveDate::parse_from_str(&event.dt, "%Y-%m-%d")?;
    let dt = chrono::NaiveDateTime::new(date, chrono::Local::now().naive_local().time());

    // Consuming is negative bottles
    let bottles = -event.bottles;
    db::add_wine_event(&state.db, wine_id, bottles, dt).await?;
    super::markup::wine_table_populated(&state).await
}

fn parse_image(image_data: &[u8]) -> anyhow::Result<image::DynamicImage> {
    let reader = image::ImageReader::new(std::io::Cursor::new(image_data)).with_guessed_format()?;
    let image = reader.decode()?;
    Ok(image)
}

fn png_encode_image(image: image::DynamicImage) -> anyhow::Result<Vec<u8>> {
    let mut image_encoded = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut image_encoded);
    image.write_to(&mut cursor, image::ImageFormat::Png)?;
    Ok(image_encoded)
}

fn convert_image(image_data: &[u8], is_iphone: bool) -> anyhow::Result<Vec<u8>> {
    let image = parse_image(image_data)?;
    let image = if is_iphone { image.rotate90() } else { image };
    let image = image.resize(512, 512, image::imageops::Gaussian);

    png_encode_image(image)
}

#[tracing::instrument(skip(state, user_agent))]
pub(crate) async fn set_wine_image(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum_extra::extract::TypedHeader(user_agent): axum_extra::extract::TypedHeader<
        headers::UserAgent,
    >,
    content_length: Option<axum_extra::extract::TypedHeader<headers::ContentLength>>,
    mut mp: axum::extract::Multipart,
) -> MDResult {
    tracing::info!("new image");

    if let Some(axum_extra::extract::TypedHeader(len)) = content_length
        && len.0 as usize > super::MAX_UPLOAD_BYTES
    {
        tracing::warn!(
            "Rejecting upload: content-length {} exceeds max {}",
            len.0,
            super::MAX_UPLOAD_BYTES
        );
        return Err(AppError::payload_too_large(anyhow::anyhow!(
            "Image upload too large (max {} bytes)",
            super::MAX_UPLOAD_BYTES
        )));
    }

    while let Some(field) = mp.next_field().await? {
        if let Some("image") = field.name() {
            let image_data = field.bytes().await?;
            if image_data.len() > super::MAX_UPLOAD_BYTES {
                tracing::warn!(
                    "Rejecting upload: field size {} exceeds max {}",
                    image_data.len(),
                    super::MAX_UPLOAD_BYTES
                );
                return Err(AppError::payload_too_large(anyhow::anyhow!(
                    "Image upload too large (max {} bytes)",
                    super::MAX_UPLOAD_BYTES
                )));
            }

            tracing::info!("Got image with size: {}", image_data.len());
            let is_iphone = user_agent.as_str().contains("iPhone");
            let image = convert_image(&image_data, is_iphone).context("Image conversion")?;
            db::set_wine_image(&state.db, wine_id, &image).await?;
        }
    }
    super::markup::wine_table_populated(&state).await
}

#[derive(Debug, serde::Deserialize)]
pub struct EditImage {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn edit_image(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::Form(edit_image): axum::Form<EditImage>,
) -> MDResult {
    tracing::info!("edit_image");
    let image_data = db::wine_image(&state.db, wine_id)
        .await?
        .ok_or(anyhow::anyhow!("No image data"))?;
    let image = parse_image(&image_data)?;
    if edit_image.w == 0 || edit_image.h == 0 {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Crop size must be non-zero"
        )));
    }
    let (img_w, img_h) = image.dimensions();
    if edit_image.x >= img_w || edit_image.y >= img_h {
        return Err(AppError::bad_request(anyhow::anyhow!(
            "Crop origin out of bounds"
        )));
    }
    let max_w = img_w - edit_image.x;
    let max_h = img_h - edit_image.y;
    let crop_w = edit_image.w.min(max_w);
    let crop_h = edit_image.h.min(max_h);
    let image = image.crop_imm(edit_image.x, edit_image.y, crop_w, crop_h);
    let image_data = png_encode_image(image)?;
    db::set_wine_image(&state.db, wine_id, &image_data).await?;

    super::markup::wine_table_populated(&state).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn wine_image(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> std::result::Result<axum::response::Response, AppError> {
    let img = db::wine_image(&state.db, wine_id).await?;
    if let Some(img) = img {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header(axum::http::header::CONTENT_TYPE, "image/png")
            .body(img.into())
            .map_err(|e| e.into())
    } else {
        axum::response::Response::builder()
            .status(axum::http::StatusCode::NOT_FOUND)
            .body(Vec::new().into())
            .map_err(|e| e.into())
    }
}
