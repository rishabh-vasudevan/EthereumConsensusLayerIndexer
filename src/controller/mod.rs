use axum::Extension;
use axum::{routing::get, Router};

pub mod indexer;
pub mod network_participations;

pub fn start_service(pool: sqlx::PgPool) -> Router {
    Router::new()
        .route("/run_indexer", get(indexer::run_indexer))
        .route(
            "/network_participation",
            get(network_participations::find_network_participation),
        )
        .route(
            "/network_participation/validator/:id",
            get(network_participations::find_network_participation_of_a_validator),
        )
        .route(
            "/network_participation/committee/:committee_id/epoch/:epoch_id",
            get(network_participations::find_network_participation_of_a_committee),
        )
        .route(
            "/network_participation/epoch/:id",
            get(network_participations::find_network_participation_of_an_epoch),
        )
        .route(
            "/get_data_about_current_state",
            get(indexer::get_data_about_current_state),
        )
        .layer(Extension(pool))
}
