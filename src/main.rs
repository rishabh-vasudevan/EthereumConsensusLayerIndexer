use sqlx::Executor;

mod controller;
mod service;
mod utils;

type Database = sqlx::PgPool;
#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:password@localhost:5432/postgres"
    )]
    pool: Database,
) -> shuttle_axum::ShuttleAxum {
    //creating a table in the database if it does not exists
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    let router = controller::start_service(pool);

    Ok(router.into())
}
