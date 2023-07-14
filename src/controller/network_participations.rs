use crate::service::network_participation_service::*;
use axum::{
    extract::Path,
    response::{IntoResponse, Json, Response},
    Extension,
};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct ParticipationResponse {
    participation: String,
}

pub async fn find_network_participation(pool: Extension<PgPool>) -> Response {
    println!("request recieved to return network participation");
    match calculate_network_participation(&pool).await {
        Ok(val) => Json(ParticipationResponse { participation: val }).into_response(),
        Err(_) => Json(ParticipationResponse {
            participation: format!("error in running api"),
        })
        .into_response(),
    }
}

pub async fn find_network_participation_of_a_validator(
    Path(validator_id): Path<String>,
    pool: Extension<PgPool>,
) -> Response {
    println!("request recieved to return network participation");
    match calculate_network_participation_of_a_validator(validator_id, &pool).await {
        Ok(val) => Json(ParticipationResponse { participation: val }).into_response(),
        Err(_) => Json(ParticipationResponse {
            participation: format!("error in running api"),
        })
        .into_response(),
    }
}

pub async fn find_network_participation_of_a_committee(
    Path((committee_id, epoch_id)): Path<(String, String)>,
    pool: Extension<PgPool>,
) -> Response {
    println!("request recieved to return network participation");
    match calculate_network_participation_of_a_committee(epoch_id, committee_id, &pool).await {
        Ok(val) => Json(ParticipationResponse { participation: val }).into_response(),
        Err(_) => Json(ParticipationResponse {
            participation: format!("error in running api"),
        })
        .into_response(),
    }
}

pub async fn find_network_participation_of_an_epoch(
    Path(epoch_id): Path<String>,
    pool: Extension<PgPool>,
) -> Response {
    println!("request recieved to return network participation");
    match calculate_network_participation_of_an_epoch(epoch_id, &pool).await {
        Ok(val) => Json(ParticipationResponse { participation: val }).into_response(),
        Err(_) => Json(ParticipationResponse {
            participation: format!("error in running api"),
        })
        .into_response(),
    }
}
