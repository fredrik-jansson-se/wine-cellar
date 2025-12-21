use maud::Markup;

struct StateInner {
    db: sqlx::SqlitePool,
}

type State = std::sync::Arc<tokio::sync::Mutex<StateInner>>;

pub async fn run(db: sqlx::SqlitePool) -> anyhow::Result<()> {
    let state = std::sync::Arc::new(tokio::sync::Mutex::new(StateInner { db }));
    let router = axum::Router::new()
        .route("/add-wine", axum::routing::get(add_wine_form))
        .route("/add-wine", axum::routing::post(add_wine_post))
        .route("/", axum::routing::get(index))
        .route("/wines/{wine_id}/grapes", axum::routing::post(wine_grapes))
        .route("/wines/{wine_id}", axum::routing::get(wine_information))
        .route("/wines", axum::routing::get(wine_table))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:20000").await?;
    tracing::info!("Starting web server");
    axum::serve(listener, router.into_make_service()).await?;
    Ok(())
}

async fn index() -> Markup {
    use maud::DOCTYPE;

    maud::html! {
        meta name="viewport" content="width=device-width, initial-scale=1";
        link href="https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/css/bootstrap.min.css"
            rel="stylesheet" integrity="sha384-sRIl4kxILFvY47J16cr9ZwB07vP4J8+LH7qKQnuqkuIAvNWLzeN8tE5YBujZqJLB"
            crossorigin="anonymous";
        script src="https://cdn.jsdelivr.net/npm/htmx.org@2.0.8/dist/htmx.min.js" {}

        body {
          div id="main" {
            div hx-get="/wines" hx-trigger="load" hx-target="#main" {}
          }
          script src="https://cdn.jsdelivr.net/npm/bootstrap@5.3.8/dist/js/bootstrap.bundle.min.js"
                integrity="sha384-FKyoEForCGlyvwx9Hj09JcYn3nv7wiPVlz7YYwJrWVcXK/BmnVDxM+D2scQbITxI"
                crossorigin="anonymous" {}
        }
    }
}

fn drink_modal(_wine_id: i64) -> Markup {
  maud::html! {
    div class="modal fade" id="exampleModal" tabindex="-1" aria-labelledby="exampleModalLabel" aria-hidden="true" {
      div class="modal-dialog" {
        div class="modal-content" {
          div class="modal-header" {
            h1 class="modal-title fs-5" id="exampleModalLabel" { "Modal title" }
            button type="button" class="btn-close" data-bs-dismiss="modal" aria-label="Close" {}
          }
          div class="modal-body" {
          }
          div class="modal-footer" {
            button type="button" class="btn btn-secondary" data-bs-dismiss="modal" { "Close" }
            button type="button" class="btn btn-primary" {"Save changes"}
          }
        }
      }
    }
  }
}

async fn wine_table(axum::extract::State(state): axum::extract::State<State>) -> Markup {
    tracing::info!("wine_table");
    let state = state.lock().await;
    let wines = crate::db::wines(&state.db).await.expect("get wines");

    let mut disp_wines = Vec::with_capacity(wines.len());
    struct MainWine {
        pub id: i64,
        pub name: String,
        pub year: i64,
        pub num_bottles: i64,
    }

    for wine in wines {
        let inv_events = crate::db::wine_inventory_events(&state.db, wine.id)
            .await
            .expect("Get inventory");
        let inventory: i64 = inv_events.iter().map(|ie| ie.bottles).sum();
        disp_wines.push(MainWine {
            id: wine.id,
            name: wine.name,
            year: wine.year,
            num_bottles: inventory,
        });
    }

    maud::html! {
        h1 {"Wine Cellar"}
        a href="#" hx-trigger="click" hx-target="#main" hx-get="/add-wine" { "Add Wine" }
        table class="table table-striped" {
            thead {
                tr {
                    th { "Name" }
                    th { "Year" }
                    th { "Bottles" }
                    th {}
                }
            }
            tbody {
                @for w in disp_wines {
                    tr {
                        td {
                            a href="#"
                              class="link-primary"
                              hx-trigger="click" hx-target="#main" hx-get=(format!("/wines/{}", w.id))
                                { (w.name)}
                        }
                        td {(w.year)}
                        td {(w.num_bottles)}
                        td {
                            div class="dropdown" {
                                button class="btn btn-secondary dropdown-toggle" type="button" data-bs-toggle="dropdown" aria-expanded="false" {
                                    "Action"
                                }
                                ul class="dropdown-menu" {
                                    li { a class="dropdown-item" { "Drink" } }
                                    li { a class="dropdown-item" { "Buy" } }
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
    let wine_grapes = crate::db::get_wine_grapes(&state.db, wine_id)
        .await
        .expect("Get wine grapes");
    let all_grapes = crate::db::get_grapes(&state.db).await.expect("Get grapes");
    maud::html! {
        h1 {(wine.name)}
        a href="/" { "Back" }
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
        h3 { "Grapes" }
        form hx-post=(format!("/wines/{}/grapes", wine_id)) hx-swap="none" {
            input type="submit" value="Set Grapes" class="btn btn-primary" {}
            @for grape in all_grapes {
                div class="form-check" {
                    @if wine_grapes.contains(&grape) {
                        input class="form-check-input" name="grapes" type="checkbox" value=(grape) id=(grape) checked;
                    } @else {
                        input class="form-check-input" name="grapes" type="checkbox" value=(grape) id=(grape);
                    }
                    label class="form-check-label" for=(grape) { (grape) }
                }
            }
        }
    }
}

async fn add_wine_form(axum::extract::State(_state): axum::extract::State<State>) -> maud::Markup {
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

async fn wine_grapes(
    axum::extract::State(state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
    axum::extract::RawForm(form): axum::extract::RawForm,
) {
    // form is b"grapes=Barbera&grapes=Gamay"

    let data = String::from_utf8(form.to_vec()).expect("Convert form to string");
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
}
