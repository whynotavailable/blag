use crate::{
    app_state::AppState,
    errors::{self, not_found},
};
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query_as};
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use whynot_errors::{html_ok, HtmlResult};

#[derive(Serialize, Debug)]
struct PageList {
    posts: Vec<PageListing>,
}

#[derive(FromRow, Serialize, Debug)]
struct PageListing {
    title: String,
    slug: String,
    description: String,
    category: String,
    category_id: Uuid,
}

#[derive(Deserialize, Debug)]
struct SearchParams {
    page: Option<i32>,
    category: Option<Uuid>,
}

// TODO: paging
async fn get_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> HtmlResult {
    let target_page: i32 = params.page.unwrap_or(0);

    let mut posts: Vec<PageListing> = query_as("SELECT * FROM list_posts(6, $1, $2)")
        .bind(target_page)
        .bind(params.category)
        .fetch_all(&state.db)
        .await
        .map_err(errors::server_error)?;

    posts.truncate(5);

    let data = PageList { posts };

    let contents = state.registry.render("list", &data).map_err(not_found)?;

    html_ok(contents)
}

#[derive(FromRow, Serialize, Debug)]
struct GetPageData {
    title: String,
    content: String,
}

#[allow(unused_variables)]
async fn get_page(State(state): State<AppState>, Path(slug): Path<String>) -> HtmlResult {
    let query: &'static str = r#"
    SELECT title, content
    FROM pages
    WHERE 
        slug = $1;
    "#;

    let data: GetPageData = query_as(query)
        .bind(slug)
        .fetch_one(&state.db)
        .await
        .map_err(errors::not_found)?;

    let contents = state.registry.render("page", &data).map_err(not_found)?;
    html_ok(contents)
}

#[derive(FromRow, Serialize, Debug)]
struct GetPostData {
    title: String,
    content: String,
    category: String,
    category_id: Uuid,
}

#[allow(unused_variables)]
async fn get_post(State(state): State<AppState>, Path(slug): Path<String>) -> HtmlResult {
    let query = r#"
    SELECT 
        posts.title, 
        posts.content,
        category.name as category,
        category.id as category_id
    FROM posts 
        INNER JOIN category
        ON posts.category = category.id
    WHERE 
        slug = $1;
    "#;

    let data: GetPostData = query_as(query)
        .bind(slug)
        .fetch_one(&state.db)
        .await
        .map_err(errors::not_found)?;

    let contents = state.registry.render("post", &data).map_err(not_found)?;
    html_ok(contents)
}

async fn lt_lock() -> HtmlResult {
    html_ok("lt")
}

pub fn ui_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_search))
        .route("/page/{slug}", get(get_page))
        .route("/post/{slug}", get(get_post))
        .route("/lt", get(lt_lock))
        .layer(CorsLayer::permissive()) // TODO: fix this lol
}
