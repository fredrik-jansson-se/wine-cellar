use super::State;
use crate::{db, web::MDResult};
use chrono::Datelike;
use maud::Markup;

pub(crate) mod image;

const TRACE_SCRIPT: &str = r#"
document.body.addEventListener("htmx:afterRequest", (e) => {
const tp = e.detail.xhr.getResponseHeader("traceparent");
if (tp) window.currentTraceparent = tp;
});

document.body.addEventListener("htmx:configRequest", (e) => {
if (window.currentTraceparent) {
  e.detail.headers["traceparent"] = window.currentTraceparent;
}
});
"#;

#[tracing::instrument]
pub(crate) async fn index() -> Markup {
    use maud::DOCTYPE;
    maud::html! {
     (DOCTYPE)
     meta name="viewport" content="width=device-width, initial-scale=1";
     meta charset="utf-8";
     link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/css/bootstrap.min.css"
         rel="stylesheet" integrity="sha384-sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB"
         crossorigin="anonymous";
     script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js" {}
     script src="https://cdn.jsdelivr.net/npm/htmx-ext-response-targets@2.0.4" integrity="sha384-T41oglUPvXLGBVyRdZsVRxNWnOOqCynaPubjUVjxhsjFTKrFJGEMm3/0KGmNQ+Pg" crossorigin="anonymous" {}
     body hx-ext="response-targets" {
       div id="main" class="container" {
         div id="error" {}
         div hx-get="/wines" hx-trigger="load" hx-target="#main" hx-target-error="#error" {}
       }
       (add_wine_modal())
       script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/js/bootstrap.bundle.min.js"
         integrity="sha384-FKyoEForCGlyvwx9Hj09JcYn3nv7wiPVlz7YYwJrWVcXK/BmnVDxM+D2scQbITxI"
         crossorigin="anonymous" {}
       script {
           (maud::PreEscaped(TRACE_SCRIPT))
       }
      }
    }
}

fn page_header(header: &str) -> Markup {
    maud::html! {
        h1 class="display-1" {(header)}
    }
}

fn add_wine_modal() -> Markup {
    let this_year = chrono::Local::now().year();
    maud::html! {
        div class="modal" id="addWineModal" tabindex="-1" {
            div class="modal-dialog" {
                div class="modal-content" {
                    div class="modal-header" {
                        h5 class="modal-title" { "Add Wine" }
                        button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" {}
                    }
                    form id="add-wine"
                        hx-post="/add-wine" hx-target="#wineTableBody" hx-swap="beforeend" hx-target-error="#error" {
                        div class="modal-body" {
                            div class="mb-3" {
                                label for="name" class="form-label" { "Name" }
                                input name="name" id="name" class="form-control" {}
                            }
                            div class="mb-3" {
                                label for="year" class="form-label" { "Year" }
                                input name="year" id="year" class="form-control" type="number" value=(this_year) {}
                            }
                        }
                        div class="modal-footer" {
                            button type="button" class="btn btn-secondary" data-bs-dismiss="modal" { "Close " }
                            button type="submit" class="btn btn-primary" data-bs-dismiss="modal" { "Add Wine" }
                        }
                    }
                }
            }
        }
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn wine_table_row(
    state: &crate::web::StateInner,
    wine: crate::db::Wine,
    grape_filter: Option<&str>,
) -> MDResult {
    tracing::info!("Rendering row for {}", wine.name);
    let wine_grapes = db::get_wine_grapes(&state.db, wine.wine_id).await?;
    if let Some(grape_filter) = grape_filter.map(|gf| gf.to_lowercase())
        && !grape_filter.is_empty()
        && !wine_grapes
            .iter()
            .any(|grape| grape.to_lowercase().starts_with(&grape_filter))
    {
        return Ok(maud::html! {});
    }

    #[derive(Debug)]
    struct MainWine {
        id: i64,
        name: String,
        year: i64,
        num_bottles: i64,
        last_comment: Option<String>,
        has_image: bool,
    }
    let inv_events = db::wine_inventory_events(&state.db, wine.wine_id).await?;
    let inventory: i64 = inv_events.iter().map(|ie| ie.bottles).sum();
    let food_pairings = db::get_wine_food_pairings(&state.db, wine.wine_id).await?;
    let w = MainWine {
        id: wine.wine_id,
        name: wine.name.clone(),
        year: wine.year,
        num_bottles: inventory,
        last_comment: wine.comment.clone(),
        has_image: wine.has_image,
    };

    Ok(maud::html! {
        tr id=(format!("wine-{}", w.id)) {
            td style="text-align: center" {
                @if w.has_image {
                    img src=(format!("/wines/{}/image", w.id)) height="80";
                }
            }
            td {
                a href="#"
                  class="link-primary"
                  hx-trigger="click" hx-target="#main" hx-target-error="#error" hx-get=(format!("/wines/{}", w.id))
                    { (w.name)}
            }
            td {(w.year)}
            td {(w.num_bottles)}
            td {
                @if let Some(comment) = w.last_comment {
                    (comment)
                }
            }
            td {
                ul {
                    @for pairing in &food_pairings {
                        li { (pairing.food) }
                    }
                }
            }
            td {
                div class="dropdown" {
                    button class="btn btn-secondary dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false" {
                        "Action"
                    }
                    ul class="dropdown-menu" {
                        li { a class="dropdown-item"
                            hx-target="#main"
                            hx-target-error="#error"
                            hx-get=(format!("/wines/{}/consume", w.id))
                            { "Consume" }
                        }

                        li { a class="dropdown-item"
                            hx-target="#main"
                            hx-target-error="#error"
                            hx-get=(format!("/wines/{}/buy", w.id))
                            { "Buy" }
                        }

                        li { a class="dropdown-item"
                            hx-target="#main"
                            hx-target-error="#error"
                            hx-get=(format!("/wines/{}/grapes", w.id))
                            { "Grapes" } }
                        li { a class="dropdown-item"
                            hx-target="#main"
                            hx-target-error="#error"
                            hx-get=(format!("/wines/{}/pairings", w.id))
                            { "Pairings" } }
                        li { a class="dropdown-item"
                            hx-trigger="click"
                            hx-target="#main"
                            hx-target-error="#error"
                            hx-get=(format!("/wines/{}/upload-image", w.id)) class="dropdown-item"
                            { "Upload Image" }}

                        li { a class="dropdown-item"
                            hx-trigger="click"
                            hx-target="#main"
                            hx-target-error="#error"
                            hx-get=(format!("/wines/{}/edit-image", w.id)) class="dropdown-item"
                            { "Edit Image" }}

                        li { a class="dropdown-item"
                            hx-target=(format!("#wine-{}", w.id))
                            hx-swap="delete"
                            hx-target-error="#error"
                            hx-delete=(format!("/wines/{}", w.id))
                            hx-confirm="Are you sure you wish to delete this wine?"
                            { "Delete" }
                        }
                    }
                }
            }
        }
    })
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct WineTableBody {
    grape_filter: Option<String>,
}

async fn render_wine_rows(state: &crate::web::StateInner, grape_filter: Option<&str>) -> MDResult {
    let wines = db::wines(&state.db).await?;
    Ok(maud::html! {
        @for wine in wines {
            (wine_table_row(state, wine, grape_filter).await?)
        }
    })
}

#[tracing::instrument(skip(state))]
pub(crate) async fn wine_table_body(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Query(query): axum::extract::Query<WineTableBody>,
) -> MDResult {
    tracing::info!("enter");
    render_wine_rows(&state, query.grape_filter.as_deref()).await
}

fn wine_table_html(body: Option<Markup>) -> Markup {
    maud::html! {
        (page_header("Wine Cellar"))
        a href="#" data-bs-toggle="modal" data-bs-target="#addWineModal" {"Add Wine"}
        " "
        a href="#"
          class="ms-2"
          hx-get="/pairings/search"
          hx-target="#main"
          hx-target-error="#error"
        { "Food Pairings Search" }
        div id="error" {}
        table class="table table-striped" {
            thead {
                tr {
                    th scope="col" {}
                    th scope="col" { "Name" }
                    th scope="col" { "Year" }
                    th scope="col" { "Bottles" }
                    th scope="col" { "Comment" }
                    th scope="col" {
                        "Pairings"
                        svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
                            fill="currentColor" class="bi bi-filter" viewBox="0 0 16 16"
                            data-bs-toggle="collapse" data-bs-target="#filterGrapes"
                        {
                          path d="M6 10.5a.5.5 0 0 1 .5-.5h3a.5.5 0 0 1 0 1h-3a.5.5 0 0 1-.5-.5m-2-3a.5.5 0 0 1 .5-.5h7a.5.5 0 0 1 0 1h-7a.5.5 0 0 1-.5-.5m-2-3a.5.5 0 0 1 .5-.5h11a.5.5 0 0 1 0 1h-11a.5.5 0 0 1-.5-.5";
                        }
                        div id="filterGrapes" class="accordion-collapse collapse" {
                            input name="grape_filter" id="grapeFilter" class="form-control"
                            hx-get="/wine-table-body"
                            hx-trigger="input changed delay:500ms, keyup[key=='Enter'],load"
                            hx-target="#wineTableBody"
                            {}
                        }
                    }
                    th scope="col" {}
                }
            }
            tbody id="wineTableBody" {
                @if let Some(body) = body {
                    (body)
                } @else {
                    div hx-get="/wine-table-body" hx-trigger="load" hx-target="#wineTableBody" hx-target-error="#error" {}
                }
            }
        }
    }
}

#[tracing::instrument]
pub(crate) async fn wine_table() -> MDResult {
    tracing::info!("wine_table");
    Ok(wine_table_html(None))
}

pub(crate) async fn wine_table_populated(state: &crate::web::StateInner) -> MDResult {
    let rows = render_wine_rows(state, None).await?;
    Ok(wine_table_html(Some(rows)))
}

#[tracing::instrument(skip(state))]
pub(crate) async fn wine_information(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    tracing::info!("enter");
    let wine = db::get_wine(&state.db, wine_id).await?;
    let events = db::wine_inventory_events(&state.db, wine_id).await?;
    Ok(maud::html! {
        (page_header(&wine.name))
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
                            td {(evt.dt.date())}
                            td {(evt.bottles)}
                        }
                    }
                  }
                }
                h3 { "Note" }
                (note_read_view(&wine))
            }
            div class="col" {
                img src=(format!("/wines/{wine_id}/image"));
            }
        }
    })
}

/// Renders the read-only note partial for a wine. Used on the detail page and returned by
/// GET /wines/{id}/comment and POST /wines/{id}/comment on success.
pub(crate) fn note_read_view(wine: &db::Wine) -> Markup {
    let wine_id = wine.wine_id;
    maud::html! {
        div id=(format!("wine-note-{wine_id}")) {
            @if let Some(ref text) = wine.comment {
                p { (text) }
                @if let Some(ref updated_at) = wine.comment_updated_at {
                    p class="text-muted small" { "Last updated: " (updated_at.date()) }
                }
            } @else {
                p class="text-muted" { "Add a note…" }
            }
            button class="btn btn-sm btn-outline-secondary"
                hx-get=(format!("/wines/{wine_id}/comment/edit"))
                hx-target=(format!("#wine-note-{wine_id}"))
                hx-swap="outerHTML"
            { "✏ Edit" }
        }
    }
}

/// Renders the inline edit form for the note. Used by GET /wines/{id}/comment/edit and
/// returned by POST /wines/{id}/comment on DB error (edit mode stays open with text preserved).
pub(crate) fn note_edit_form(wine_id: i64, current_text: &str, error: Option<&str>) -> Markup {
    maud::html! {
        div id=(format!("wine-note-{wine_id}")) {
            @if let Some(msg) = error {
                div class="alert alert-danger" role="alert" { (msg) }
            }
            form
                hx-post=(format!("/wines/{wine_id}/comment"))
                hx-target=(format!("#wine-note-{wine_id}"))
                hx-swap="outerHTML"
            {
                textarea name="comment" class="form-control mb-2" rows="4" { (current_text) }
                button type="submit" class="btn btn-sm btn-primary me-2" { "Save" }
                button type="button" class="btn btn-sm btn-secondary"
                    hx-get=(format!("/wines/{wine_id}/comment"))
                    hx-target=(format!("#wine-note-{wine_id}"))
                    hx-swap="outerHTML"
                { "Cancel" }
            }
        }
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn edit_wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    let wine_grapes = db::get_wine_grapes(&state.db, wine_id).await?;
    let all_grapes = db::get_grapes(&state.db).await?;
    Ok(maud::html! {
        (page_header("Grapes"))
        div id="error" {}
        form hx-post=(format!("/wines/{}/grapes", wine_id))
            hx-target="#main"
            hx-target-error="#error"
            {
            div class="mb-3" {
                input type="submit" value="Set Grapes" class="btn btn-primary me-3" {}
                button hx-trigger="click" hx-target="#main" hx-get="/" class="btn btn-secondary" {
                    "Cancel"
                }
            }
            @for grape in all_grapes {
                @let id=format!("grape-{}", grape.rowid);
                div class="form-check" {
                    input class="form-check-input" name=(id) type="checkbox" value=(grape.name) id=(id) checked[(wine_grapes.contains(&grape.name))];
                    label class="form-check-label" for=(id) { (grape.name) };
                }
            }
        }
    })
}

pub(crate) async fn consume_wine(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    tracing::info!("consume_wine");
    let today = chrono::Local::now().date_naive();
    maud::html! {
        (page_header("Consume Wine"))
        div id="error" {}
        form id="consume-wine"
            hx-post=(format!("/wines/{wine_id}/consume"))
            hx-target="#main"
            hx-target-error="#error" {
            div class="mb-3" {
                label for="dt" class="form-label" { "Date" }
                input name="dt" id="dt" type="date" class="form-control" value=(today) {}
            }
            div class="mb-3" {
                label for="bottles" class="form-label" { "Bottles" }
                input name="bottles" id="bottles" type="number" value="1" class="form-control" {}
            }
            div class="mb-3" {
                input type="submit" value="Consume" class="btn btn-primary me-3" {}
                button hx-trigger="click" hx-target="#main" hx-get="/wines" class="btn btn-secondary" {
                    "Cancel"
                }
            }
        }
    }
}
pub(crate) async fn buy_wine(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    tracing::info!("buy_wine");
    let today = chrono::Local::now().date_naive();
    maud::html! {
        (page_header("Buy Wine"))
        div id="error" {}
        form id="buy-wine" hx-post=(format!("/wines/{wine_id}/buy"))
            hx-target="#main"
            hx-target-error="#error" {
            div class="mb-3" {
                label for="dt" class="form-label" { "Date" }
                input name="dt" id="dt" type="date" class="form-control" value=(today) {}
            }
            div class="mb-3" {
                label for="bottles" class="form-label" { "Bottles" }
                input name="bottles" id="bottles" type="number" value="6" class="form-control" {}
            }
            div class="mb-3" {
                input type="submit" value="Buy" class="btn btn-primary me-3" {}
                button hx-trigger="click" hx-target="#main" hx-get="/wines" class="btn btn-secondary" {
                    "Cancel"
                }
            }
        }
    }
}

pub(crate) async fn upload_wine_image(
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> Markup {
    maud::html! {
        (page_header("Upload Image"))
        div id="error" {}
        form hx-encoding="multipart/form-data"
            hx-target="#main"
            hx-target-error="#error"
            hx-post=(format!("/wines/{wine_id}/image")) {
           input type="file" name="image";
           input type="submit" value="Upload" class="btn btn-primary" {}
        }
    }
}

// ── Food Pairings ────────────────────────────────────────────────────────────

/// Renders the `<li>` items for the food pairings list (partial used by add/remove handlers).
/// The caller is responsible for the outer `<ul id="food-pairings-list">`.
pub(crate) fn food_pairings_list_items(pairings: &[db::FoodPairing], wine_id: i64) -> Markup {
    maud::html! {
        @if pairings.is_empty() {
            li class="list-group-item text-muted" { "No pairings added yet." }
        } @else {
            @for pairing in pairings {
                li class="list-group-item d-flex justify-content-between align-items-center"
                   id=(format!("pairing-{}", pairing.id))
                {
                    span { (pairing.food) }
                    button
                        class="btn btn-sm btn-outline-danger"
                        hx-delete=(format!("/wines/{}/pairings/{}", wine_id, pairing.id))
                        hx-target="#food-pairings-list"
                        hx-swap="innerHTML"
                        hx-target-error="#error"
                        hx-confirm="Remove this pairing?"
                    { "Remove" }
                }
            }
        }
    }
}

#[tracing::instrument(skip(state))]
pub(crate) async fn edit_wine_pairings(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    let wine = db::get_wine(&state.db, wine_id).await?;
    let pairings = db::get_wine_food_pairings(&state.db, wine_id).await?;
    Ok(maud::html! {
        (page_header(&format!("Food Pairings — {}", wine.name)))
        div id="error" {}
        button class="btn btn-secondary mb-3"
            hx-get="/wines"
            hx-target="#main"
            hx-trigger="click"
        { "← Back" }
        ul id="food-pairings-list" class="list-group mb-3" {
            (food_pairings_list_items(&pairings, wine_id))
        }
        form
            hx-post=(format!("/wines/{}/pairings", wine_id))
            hx-target="#food-pairings-list"
            hx-swap="innerHTML"
            hx-target-error="#error"
            class="d-flex gap-2"
        {
            input
                name="food"
                type="text"
                class="form-control"
                placeholder="e.g. grilled salmon"
                maxlength="100"
                required
            {}
            button type="submit" class="btn btn-primary" { "Add Pairing" }
        }
    })
}

// ── Pairings Search ──────────────────────────────────────────────────────────

pub(crate) fn pairings_search_page() -> Markup {
    maud::html! {
        (page_header("Food Pairings Search"))
        div id="error" {}
        button class="btn btn-secondary mb-3"
            hx-get="/wines"
            hx-target="#main"
            hx-trigger="click"
        { "← Back" }
        div class="mb-3" {
            input
                name="q"
                type="text"
                class="form-control"
                placeholder="Enter a food to find matching wines"
                hx-get="/pairings/search/results"
                hx-target="#search-results"
                hx-swap="innerHTML"
                hx-target-error="#error"
                hx-trigger="input changed delay:500ms, keyup[key=='Enter']"
            {}
        }
        div id="search-results" {
            (pairings_search_prompt())
        }
    }
}

pub(crate) fn pairings_search_prompt() -> Markup {
    maud::html! {
        p class="text-muted" { "Enter a food to find matching wines." }
    }
}

pub(crate) fn pairings_search_results_markup(wines: &[db::WineWithPairings], q: &str) -> Markup {
    maud::html! {
        @if wines.is_empty() {
            p class="text-muted" { "No wines found matching \"" (q) "\"." }
        } @else {
            @for wine in wines {
                div class="card mb-2" {
                    div class="card-body" {
                        h5 class="card-title" {
                            (wine.name)
                            " "
                            span class="text-muted fw-normal" { "(" (wine.year) ")" }
                        }
                        p class="card-text" {
                            "Pairings: "
                            @for (i, pairing) in wine.matched_pairings.iter().enumerate() {
                                @if i > 0 { ", " }
                                (pairing)
                            }
                        }
                    }
                }
            }
        }
    }
}
