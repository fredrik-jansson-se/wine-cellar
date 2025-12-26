use super::{MDResult, State};
use anyhow::Context;
use base64::Engine;

use crate::db;

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
    {
        let state = state.lock().await;
        db::add_wine(&state.db, &wine.name, wine.year).await?;
    }
    super::markup::wine_table(axum::extract::State(state)).await
}

#[tracing::instrument(skip(state))]
pub(crate) async fn delete_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    tracing::info!("Delete");
    {
        let state = state.lock().await;

        db::delete_wine(&state.db, wine_id).await?;
    }
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

    {
        let state = state.lock().await;
        db::set_wine_grapes(&state.db, wine_id, &grapes).await?;
    }
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
    {
        let state = state.lock().await;
        let now = chrono::Local::now().naive_local();
        db::add_wine_comment(&state.db, wine_id, &comment.comment, now).await?;
    }
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
    {
        let state = state.lock().await;
        let date = chrono::NaiveDate::parse_from_str(&event.dt, "%Y-%m-%d")?;
        let dt = chrono::NaiveDateTime::new(date, chrono::Local::now().naive_local().time());
        db::add_wine_event(&state.db, wine_id, event.bottles, dt).await?;
    }
    super::markup::wine_table(axum::extract::State(state)).await
}

#[derive(serde::Deserialize, Debug)]
pub(crate) struct DrinkWine {
    dt: String,
    bottles: i64,
}

#[tracing::instrument(skip(state))]
pub(crate) async fn drink_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(event): axum::extract::Form<DrinkWine>,
) -> MDResult {
    tracing::info!("drink wine");
    {
        let state = state.lock().await;
        let date = chrono::NaiveDate::parse_from_str(&event.dt, "%Y-%m-%d")?;
        let dt = chrono::NaiveDateTime::new(date, chrono::Local::now().naive_local().time());

        // Drinking is negative bottles
        let bottles = -event.bottles;
        db::add_wine_event(&state.db, wine_id, bottles, dt).await?;
    }
    super::markup::wine_table(axum::extract::State(state)).await
}

fn convert_and_thumbnail(image_data: &[u8]) -> anyhow::Result<(String, String)> {
    let reader = image::ImageReader::new(std::io::Cursor::new(image_data)).with_guessed_format()?;

    let image = reader.decode()?;
    let thumbnail = image.thumbnail(160, 160);
    let image = image.resize(512, 512, image::imageops::Gaussian);

    let mut image_encoded = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut image_encoded);
    image.write_to(&mut cursor, image::ImageFormat::Png)?;

    let mut thumbnail_encoded = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut thumbnail_encoded);
    thumbnail.write_to(&mut cursor, image::ImageFormat::Png)?;

    let img_64 = base64::prelude::BASE64_STANDARD_NO_PAD.encode(&image_encoded);
    let tn_u64 = base64::prelude::BASE64_STANDARD_NO_PAD.encode(&thumbnail_encoded);

    Ok((img_64, tn_u64))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn set_wine_image(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    mut mp: axum::extract::Multipart,
) -> MDResult {
    tracing::info!("new image");

    while let Some(field) = mp.next_field().await? {
        if let Some("image") = field.name() {
            let image_data = field.bytes().await?;

            tracing::info!("Got image with size: {}", image_data.len());
            let (image, thumbnail) =
                convert_and_thumbnail(&image_data).context("Image conversion")?;
            let state = state.lock().await;
            db::set_wine_image(&state.db, wine_id, &image, &thumbnail).await?;
        }
    }
    super::markup::wine_table(axum::extract::State(state)).await
}
