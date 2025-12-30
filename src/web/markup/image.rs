use crate::web::{MDResult, State};

const EDIT_IMG_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/js/edit-image.js"));
const STYLE: &str = r#"
            .img-select-wrap {
                position: relative;
                display: inline-block;
                user-select: none;
            }
            .img-select-wrap img {
                display: block;
                max-width: "100%";
            }
            .selection-rect {
                position: absolute;
                border: 2px solid #00a3ff;
                background: rgba(0, 163, 255, 0.15);
                pointer-events: none;
                display: none;
            }
            .roi-actions { margin-top: 8px; display: flex; gap: 8px; align-items: center; }
            .roi-hint { opacity: 0.75; font-size: 0.9em; }
        "#;

// #[tracing::instrument(skip(state))]
pub(crate) async fn edit_image(
    axum::extract::State(_state): axum::extract::State<State>,
    axum::extract::Path(wine_id): axum::extract::Path<i64>,
) -> MDResult {
    Ok(maud::html! {
        style {
            (maud::PreEscaped(STYLE))
        }
        div class="img-select-wrap" id="imgWrap" {
            img id="targetImg"
                src=(format!("/wines/{wine_id}/image"))
                alt="Select region"
                draggable="false";
            div id="rect" class="selection-rect" {}
        }

        form id="roiForm"
            hx-post=(format!("/wines/{wine_id}/edit-image"))
            hx-target="#main" {
                input type="hidden" name="x" id="roiX";
                input type="hidden" name="y" id="roiY";
                input type="hidden" name="w" id="roiW";
                input type="hidden" name="h" id="roiH";
                input type="hidden" name="imageId" value="example.jpg";

                div class="roi-actions" {
                    button type="submit" id="submitRoi" class="btn btn-primary" disabled {"Submit"}
                    button type="button" id="clearRoi" class="btn btn-secondary" disabled {"Clear"}
                    span class="roi-hint" id="roiHint"{"Drag on the image to select a region."}
                }
            }

        div id="roiResult" {}
        script {(maud::PreEscaped(EDIT_IMG_JS))}
    })
}
