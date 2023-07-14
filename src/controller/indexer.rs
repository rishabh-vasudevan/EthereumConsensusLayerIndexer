use crate::service;
use axum::{
    response::{IntoResponse, Json, Response},
    Extension,
};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct IndexerResponse {
    pub status: String,
}

pub async fn run_indexer(pool: Extension<PgPool>) -> Response {
    println!("recieved request to run the indexer");

    match service::indexer_service::run_indexer_impl(pool).await {
        Ok(()) => Json(IndexerResponse {
            status: String::from("indexer ran successfully"),
        })
        .into_response(),
        Err(err) => Json(IndexerResponse {
            status: format!("indexer had an error : {}", err),
        })
        .into_response(),
    }
}

#[derive(Serialize)]
pub struct CurrentUniqueData {
    pub epochs: Vec<String>,
    pub slots: Vec<String>,
    pub validators: Vec<String>,
}

pub async fn get_data_about_current_state(pool: Extension<PgPool>) -> Response {
    println!("recieved request to get data for current state");

    match service::indexer_service::get_data_about_current_state(&pool).await {
        Ok(val) => Json(val).into_response(),
        Err(e) => {
            panic!("did not get unique values {}", e);
        }
    }
}
