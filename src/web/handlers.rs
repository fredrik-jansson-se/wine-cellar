use super::{MDResult, State};
use anyhow::Context;
use image::GenericImageView;

use crate::{db, web::AppError};

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
    super::markup::wine_table_row(&state, wine).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn delete_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    tracing::info!("Delete");
    db::delete_wine(&state.db, wine_id).await?;
    super::markup::wine_table(axum::extract::State(state)).await
}

pub(crate) async fn post_wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::RawForm(form): axum::extract::RawForm,
) -> MDResult {
    // form is b"grapes=Barbera&grapes=Gamay"
    let data = percent_encoding::percent_decode(&form).decode_utf8()?;

    tracing::info!("Data: {data}");
    let mut grapes = Vec::new();
    for grape in data.split("&") {
        let item = grape.split("=").nth(1);
        if let Some(i) = item {
            grapes.push(i);
        }
    }

    db::set_wine_grapes(&state.db, wine_id, &grapes).await?;
    super::markup::wine_table(axum::extract::State(state)).await
}

#[derive(serde::Deserialize)]
pub(crate) struct AddComment {
    comment: String,
}

pub(crate) async fn add_comment(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(comment): axum::extract::Form<AddComment>,
) -> MDResult {
    let now = chrono::Local::now().naive_local();
    db::add_wine_comment(&state.db, wine_id, &comment.comment, now).await?;
    super::markup::wine_table(axum::extract::State(state)).await
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
    super::markup::wine_table(axum::extract::State(state)).await
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
    super::markup::wine_table(axum::extract::State(state)).await
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
    content_length: Option<
        axum_extra::extract::TypedHeader<headers::ContentLength>,
    >,
    mut mp: axum::extract::Multipart,
) -> MDResult {
    tracing::info!("new image");

    if let Some(axum_extra::extract::TypedHeader(len)) = content_length {
        if len.0 as usize > super::MAX_UPLOAD_BYTES {
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
    super::markup::wine_table(axum::extract::State(state)).await
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

    super::markup::wine_table(axum::extract::State(state)).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn wine_image(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> std::result::Result<Vec<u8>, AppError> {
    Ok(db::wine_image(&state.db, wine_id)
        .await?
        .unwrap_or_default())
}
