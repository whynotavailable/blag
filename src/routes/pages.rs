use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use whynot_errors::{json_ok, AppError, JsonResult};

use crate::{app_state::AppState, auth::Auth, errors, models::SimpleResponse};

#[derive(FromRow, Serialize, Debug)]
pub struct PageListItem {
    slug: String,
    title: String,
}

pub async fn page_list(
    State(state): State<AppState>,
    Auth(_sub): Auth,
) -> JsonResult<Vec<PageListItem>> {
    let sql = "SELECT slug, title FROM pages;";

    json_ok(
        query_as(sql)
            .fetch_all(&state.db)
            .await
            .map_err(AppError::new)?,
    )
}

// Will be used for both sides.
#[derive(FromRow, Serialize, Deserialize, Debug)]
pub struct PageEdit {
    title: String,
    raw: String,
}

pub async fn page_get(
    State(state): State<AppState>,
    Auth(_sub): Auth,
    Path(slug): Path<String>,
) -> JsonResult<PageEdit> {
    let sql = "SELECT title, raw FROM pages WHERE slug = $1;";

    json_ok(
        query_as(sql)
            .bind(slug)
            .fetch_one(&state.db)
            .await
            .map_err(errors::not_found)?,
    )
}

pub async fn page_update(
    State(state): State<AppState>,
    Auth(_sub): Auth,
    Path(slug): Path<String>,
    Json(body): Json<PageEdit>,
) -> JsonResult<SimpleResponse> {
    let sql = "UPDATE pages SET title = $1, raw = $2, content = $3 WHERE slug = $4;";
    let html = markdown::to_html(&body.raw);

    query(sql)
        .bind(body.title)
        .bind(body.raw)
        .bind(html)
        .bind(slug)
        .execute(&state.db)
        .await
        .map_err(AppError::new)?;

    SimpleResponse::json("ok")
}
