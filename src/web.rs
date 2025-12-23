use axum::response::IntoResponseParts;
use base64::Engine;
use maud::Markup;

use crate::db;

struct StateInner {
    db: sqlx::SqlitePool,
}

type State = std::sync::Arc<tokio::sync::Mutex<StateInner>>;

pub async fn run(db: sqlx::SqlitePool) -> anyhow::Result<()> {
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(StateInner { db }));
    let router = axum::Router::new()
        .route("/add-wine", axum::routing::get(md_add_wine))
        .route("/add-wine", axum::routing::post(add_wine_post))
        .route("/", axum::routing::get(index))
        .route(
            "/wines/{wine_id}/image",
            axum::routing::get(md_upload_wine_image),
        )
        .route(
            "/wines/{wine_id}/image",
            axum::routing::post(post_wine_image)
                // Disable body limit on file
                .layer(axum::extract::DefaultBodyLimit::disable()),
        )
        .route(
            "/wines/{wine_id}/comment",
            axum::routing::get(md_add_comment),
        )
        .route("/wines/{wine_id}/comment", axum::routing::post(add_comment))
        .route(
            "/wines/{wine_id}/grapes",
            axum::routing::get(edit_wine_grapes),
        )
        .route(
            "/wines/{wine_id}/grapes",
            axum::routing::post(post_wine_grapes),
        )
        .route("/wines/{wine_id}", axum::routing::get(wine_information))
        .route("/wines/{wine_id}", axum::routing::delete(delete_wine))
        .route("/wines", axum::routing::get(wine_table))
        .with_state(state);

    let lap = std::env::var("WINE_LAP").unwrap_or("0.0.0.0:20000".to_owned());
    let listener = tokio::net::TcpListener::bind(lap).await?;
    tracing::info!("Starting web server");
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

async fn index() -> Markup {
    use maud::DOCTYPE;
    maud::html! {
     (DOCTYPE)
     meta name="viewport" content="width=device-width, initial-scale=1";
     meta charset="utf-8";
     link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/css/bootstrap.min.css"
         rel="stylesheet" integrity="sha384-sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB"
         crossorigin="anonymous";
     script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js" {}

     body {
       div id="main" class="container" {
         div hx-get="/wines" hx-trigger="load" hx-target="#main" {}
       }
       script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/js/bootstrap.bundle.min.js"
         integrity="sha384-FKyoEForCGlyvwx9Hj09JcYn3nv7wiPVlz7YYwJrWVcXK/BmnVDxM+D2scQbITxI"
         crossorigin="anonymous" {}
      }
    }
}

async fn wine_table(axum::extract::State(state): axum::extract::State<State>) -> Markup {
    tracing::info!("wine_table");
    let state = state.lock().await;
    let wines = crate::db::wines(&state.db).await.expect("get wines");

    let mut disp_wines = Vec::with_capacity(wines.len());
    struct MainWine {
        id: i64,
        name: String,
        year: i64,
        num_bottles: i64,
        thumbnail: Option<String>,
        grapes: Vec<String>,
    }

    for wine in wines {
        let inv_events = crate::db::wine_inventory_events(&state.db, wine.id)
            .await
            .expect("Get inventory");
        let inventory: i64 = inv_events.iter().map(|ie| ie.bottles).sum();
        let wine_grapes = crate::db::get_wine_grapes(&state.db, wine.id)
            .await
            .expect("Get wine grapes");
        disp_wines.push(MainWine {
            id: wine.id,
            name: wine.name,
            year: wine.year,
            num_bottles: inventory,
            thumbnail: wine.image_thumbnail_b64,
            grapes: wine_grapes,
        });
    }

    maud::html! {
        h1 {"Wine Cellar"}
        a href="#" hx-trigger="click" hx-target="#main" hx-get="/add-wine" { "Add Wine" }
        table class="table table-striped" {
            thead {
                tr {
                    th {}
                    th { "Name" }
                    th { "Year" }
                    th { "Bottles" }
                    th { "Grapes" }
                    th {}
                }
            }
            tbody {
                @for w in disp_wines {
                    tr {
                        td {
                            @if let Some(tn) = &w.thumbnail {
                                img src=(format!("data:image/png;base64, {tn}"));

                            }
                        }
                        td {
                            a href="#"
                              class="link-primary"
                              hx-trigger="click" hx-target="#main" hx-get=(format!("/wines/{}", w.id))
                                { (w.name)}
                        }
                        td {(w.year)}
                        td {(w.num_bottles)}
                        td {
                            ul {
                                @for grape in w.grapes {
                                    li {
                                        (grape)
                                    }
                                }
                            }
                        }
                        td {
                            div class="dropdown" {
                                button class="btn btn-secondary dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false" {
                                    "Action"
                                }
                                ul class="dropdown-menu" {
                                    li { a class="dropdown-item" { "Drink" } }
                                    li { a class="dropdown-item" { "Buy" } }
                                    li { a class="dropdown-item" hx-target="#main" hx-get=(format!("/wines/{}/comment", w.id))  { "Comment" } }
                                    li { a class="dropdown-item" hx-target="#main" hx-get=(format!("/wines/{}/grapes", w.id))  { "Grapes" } }
                                    li { a hx-trigger="click" hx-target="#main" hx-get=(format!("/wines/{}/image", w.id)) class="dropdown-item" { "Upload Image" }}
                                    li { a hx-target="#main"
                                           hx-delete=(format!("/wines/{}", w.id))
                                           hx-confirm="Are you sure you wish to delete this wine?"
                                           class="dropdown-item" { "Delete" }}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[tracing::instrument(skip(state))]
async fn wine_information(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> Markup {
    tracing::info!("enter");
    let state = state.lock().await;
    let wine = crate::db::get_wine(&state.db, wine_id)
        .await
        .expect("Get wine info");
    let events = crate::db::wine_inventory_events(&state.db, wine_id)
        .await
        .expect("Get events");
    let comments = crate::db::wine_comments(&state.db, wine_id)
        .await
        .expect("Get wine comments");
    maud::html! {
        h1 {(wine.name)}
        a href="/" { "Back" }
        div class="row align-items-start" {
            div class="col" {
                h3 { "Events" }
                table class="table table-striped" {
                  thead {
                      tr {
                          th { "Date" }
                          th { "Bottles" }
                      }
                  }
                  tbody {
                    @for evt in events {
                        tr {
                            td {(evt.dt)}
                            td {(evt.bottles)}
                        }
                    }
                  }
                }
                h3 { "Comments" }
                @for comment in comments {
                    h4 { (comment.dt.date()) }
                    p { (comment.comment) }
                }
            }
            div class="col" {
                @if let Some(image) = &wine.image_b64 {
                    img src=(format!("data:image/png;base64, {image}"));
                }
            }
        }
    }
}

async fn md_add_wine(axum::extract::State(_state): axum::extract::State<State>) -> maud::Markup {
    tracing::info!("Adding wine");
    maud::html! {
        h1 { "Add wine" }
        form id="add-wine" hx-post="/add-wine" hx-target="#main" {
            div class="mb-3" {
                label for="name" class="form-label" { "Name" }
                input name="name" id="name" class="form-control" {}
            }
            div class="mb-3" {
                label for="year" class="form-label" { "Year" }
                input name="year" id="year" class="form-control" type="number" {}
            }
            div class="mb-3" {
                input type="submit" value="Add" class="btn btn-primary" {}
                button hx-trigger="click" hx-target="#main" hx-swap="innerHTML" hx-get="/" class="btn btn-primary" {
                    "Cancel"
                }
            }
        }

    }
}

#[derive(serde::Deserialize, Debug)]
struct AddWine {
    name: String,
    year: i64,
}

async fn add_wine_post(
    axum::extract::State(state): axum::extract::State<State>,
    axum::Form(wine): axum::Form<AddWine>,
) -> maud::Markup {
    tracing::info!("Adding wine: {wine:?}");
    {
        let state = state.lock().await;
        crate::db::add_wine(&state.db, &wine.name, wine.year)
            .await
            .expect("Add wine");
    }
    wine_table(axum::extract::State(state)).await
}

async fn delete_wine(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> (impl IntoResponseParts, &'static str) {
    let state = state.lock().await;

    db::delete_wine(&state.db, wine_id)
        .await
        .expect("Delete wine");
    (axum_htmx::HxRedirect("/".to_owned()), "")
}

async fn edit_wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> Markup {
    let state = state.lock().await;
    let wine_grapes = crate::db::get_wine_grapes(&state.db, wine_id)
        .await
        .expect("Get wine grapes");
    let all_grapes = crate::db::get_grapes(&state.db).await.expect("Get grapes");
    maud::html! {
        form hx-post=(format!("/wines/{}/grapes", wine_id)) hx-swap="none" {
            input type="submit" value="Set Grapes" class="btn btn-primary" {}
            @for grape in all_grapes {
                div class="form-check" {
                    // @if wine_grapes.contains(&grape) {
                    input class="form-check-input" name="grapes" type="checkbox" value=(grape) id=(grape) checked[(wine_grapes.contains(&grape))]
                        // } @else {
                        // input class="form-check-input" name="grapes" type="checkbox" value=(grape) id=(grape);
                        // }
                        label class="form-check-label" for=(grape) { (grape) }
                }
            }
        }
    }
}

async fn post_wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::RawForm(form): axum::extract::RawForm,
) -> (impl IntoResponseParts, &'static str) {
    // form is b"grapes=Barbera&grapes=Gamay"
    let data = percent_encoding::percent_decode(&form)
        .decode_utf8()
        .expect("Valid utf-8");

    tracing::info!("Data: {data}");
    let mut grapes = Vec::new();
    for grape in data.split("&") {
        let item = grape.split("=").nth(1);
        if let Some(i) = item {
            grapes.push(i);
        }
    }

    let state = state.lock().await;
    crate::db::set_wine_grapes(&state.db, wine_id, &grapes)
        .await
        .expect("Can set grapes");
    (axum_htmx::HxRedirect("/".to_owned()), "")
}

async fn md_add_comment(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    maud::html! {
        form id="add-comment" hx-post=(format!("/wines/{wine_id}/comment")) hx-target="#main" {
            div class="mb-3" {
                label for="comment" class="form-label" { "Comment" }
                input name="comment" id="comment" class="form-control" {}
            }
            div class="mb-3" {
                input type="submit" value="Add" class="btn btn-primary" {}
                button hx-trigger="click" hx-target="#main" hx-swap="innerHTML" hx-get="/" class="btn btn-primary" {
                    "Cancel"
                }
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct AddComment {
    comment: String,
}
async fn add_comment(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::Form(comment): axum::extract::Form<AddComment>,
) -> (impl IntoResponseParts, &'static str) {
    let state = state.lock().await;
    let now = chrono::Local::now().naive_local();
    db::add_wine_comment(&state.db, wine_id, &comment.comment, now)
        .await
        .expect("Add comment");
    (axum_htmx::HxRedirect("/".to_owned()), "")
}

async fn md_upload_wine_image(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    maud::html! {
        h1 { "Upload image" }
        form hx-encoding="multipart/form-data" hx-post=(format!("/wines/{wine_id}/image")) {
           input type="file" name="image";
           input type="submit" value="Upload" class="btn btn-primary" {}
        }
    }
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
async fn post_wine_image(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    mut mp: axum::extract::Multipart,
) -> (impl IntoResponseParts, &'static str) {
    tracing::info!("new image");

    while let Some(field) = mp.next_field().await.expect("Get next multipart field") {
        if let Some("image") = field.name() {
            let image_data = field.bytes().await.expect("Get image data");

            tracing::info!("Got image with size: {}", image_data.len());
            let (image, thumbnail) = convert_and_thumbnail(&image_data).expect("Convert image");
            let state = state.lock().await;
            db::set_wine_image(&state.db, wine_id, &image, &thumbnail)
                .await
                .expect("Update image in db");
        }
    }
    (axum_htmx::HxRedirect("/".to_owned()), "")
}
