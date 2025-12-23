use super::State;
use crate::db;
use maud::Markup;

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

pub(crate) async fn wine_table(axum::extract::State(state): axum::extract::State<State>) -> Markup {
    tracing::info!("wine_table");
    let state = state.lock().await;
    let wines = db::wines(&state.db).await.expect("get wines");

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
        let inv_events = db::wine_inventory_events(&state.db, wine.id)
            .await
            .expect("Get inventory");
        let inventory: i64 = inv_events.iter().map(|ie| ie.bottles).sum();
        let wine_grapes = db::get_wine_grapes(&state.db, wine.id)
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
                                    li { a class="dropdown-item" 
                                        hx-target="#main" 
                                        hx-get=(format!("/wines/{}/drink", w.id))  
                                        { "Drink" } 
                                    }

                                    li { a class="dropdown-item" 
                                        hx-target="#main" 
                                        hx-get=(format!("/wines/{}/buy", w.id))  
                                        { "Buy" } 
                                    }

                                    li { a class="dropdown-item" 
                                        hx-target="#main" 
                                        hx-get=(format!("/wines/{}/comment", w.id))  
                                        { "Comment" } 
                                    }

                                    li { a class="dropdown-item" hx-target="#main" hx-get=(format!("/wines/{}/grapes", w.id))  { "Grapes" } }
                                    li { a hx-trigger="click" hx-target="#main" hx-get=(format!("/wines/{}/image", w.id)) class="dropdown-item" { "Upload Image" }}

                                    li { a class="dropdown-item" 
                                        hx-target="#main"
                                        hx-delete=(format!("/wines/{}", w.id))
                                        hx-confirm="Are you sure you wish to delete this wine?"
                                        { "Delete" }
                                    }
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
pub(crate) async fn wine_information(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> Markup {
    tracing::info!("enter");
    let state = state.lock().await;
    let wine = db::get_wine(&state.db, wine_id)
        .await
        .expect("Get wine info");
    let events = db::wine_inventory_events(&state.db, wine_id)
        .await
        .expect("Get events");
    let comments = db::wine_comments(&state.db, wine_id)
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

pub(crate) async fn edit_wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> Markup {
    let state = state.lock().await;
    let wine_grapes = db::get_wine_grapes(&state.db, wine_id)
        .await
        .expect("Get wine grapes");
    let all_grapes = db::get_grapes(&state.db).await.expect("Get grapes");
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

pub(crate) async fn add_wine(axum::extract::State(_state): axum::extract::State<State>) -> maud::Markup {
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
                button hx-trigger="click" hx-target="#main" hx-get="/" class="btn btn-primary" {
                    "Cancel"
                }
            }
        }

    }
}

pub(crate) async fn add_comment(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    maud::html! {
        form id="add-comment" hx-post=(format!("/wines/{wine_id}/comment")) hx-target="#main" {
            div class="mb-3" {
                label for="comment" class="form-label" { "Comment" }
                input name="comment" id="comment" class="form-control" {}
            }
            div class="mb-3" {
                input type="submit" value="Add" class="btn btn-primary" {}
                button hx-trigger="click" hx-target="#main" hx-get="/" class="btn btn-primary" {
                    "Cancel"
                }
            }
        }
    }
}

pub(crate) async fn drink_wine(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    tracing::info!("drink_wine");
    maud::html! {
        form id="drink-wine" hx-post=(format!("/wines/{wine_id}/drink")) hx-target="#main" {
            div class="mb-3" {
                label for="dt" class="form-label" { "Date" }
                input name="dt" id="dt" type="date" class="form-control" {}
            }
            div class="mb-3" {
                label for="bottles" class="form-label" { "Bottles" }
                input name="bottles" id="bottles" type="number" value="1" class="form-control" {}
            }
            div class="mb-3" {
                input type="submit" value="Drink" class="btn btn-primary" {}
                button hx-trigger="click" hx-target="#main" hx-get="/wines" class="btn btn-primary" {
                    "Cancel"
                }
            }
        }
    }
}
pub(crate) async fn buy_wine(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    tracing::info!("buy_wine");
    maud::html! {
        form id="buy-wine" hx-post=(format!("/wines/{wine_id}/buy")) hx-target="#main" {
            div class="mb-3" {
                label for="dt" class="form-label" { "Date" }
                input name="dt" id="dt" type="date" class="form-control" {}
            }
            div class="mb-3" {
                label for="bottles" class="form-label" { "Bottles" }
                input name="bottles" id="bottles" type="number" class="form-control" {}
            }
            div class="mb-3" {
                input type="submit" value="Buy" class="btn btn-primary" {}
                button hx-trigger="click" hx-target="#main" hx-get="/wines" class="btn btn-primary" {
                    "Cancel"
                }
            }
        }
    }
}

pub(crate) async fn upload_wine_image(axum::extract::Path(wine_id): axum::extract::Path<i64>) -> Markup {
    maud::html! {
        h1 { "Upload image" }
        form hx-encoding="multipart/form-data" hx-post=(format!("/wines/{wine_id}/image")) {
           input type="file" name="image";
           input type="submit" value="Upload" class="btn btn-primary" {}
        }
    }
}
