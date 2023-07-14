use axum::Extension;
use sqlx::PgPool;
use std::error::Error;

#[derive(sqlx::FromRow)]
struct Count {
    count: i64,
}

pub async fn calculate_network_participation(
    pool: &Extension<PgPool>,
) -> Result<String, Box<dyn Error>> {
    let total_count_of_network: Count =
        sqlx::query_as(r#"SELECT count(*) as count FROM ATTESTATIONS"#)
            .fetch_one(&**pool)
            .await
            .map_err(|e| println!("{}", e))
            .expect("could not fetch network participation");

    let active_attestations: Count =
        sqlx::query_as(r#"SELECT count(*) as count FROM ATTESTATIONS where attested= true"#)
            .fetch_one(&**pool)
            .await
            .map_err(|e| println!("{}", e))
            .expect("could not fetch network participation");

    Ok((active_attestations.count as f64 / total_count_of_network.count as f64).to_string())
}

pub async fn calculate_network_participation_of_a_validator(
    validator_id: String,
    pool: &Extension<PgPool>,
) -> Result<String, Box<dyn Error>> {
    let total_number_of_committess_of_validator: Count =
        sqlx::query_as(r#"SELECT count(*) as count FROM ATTESTATIONS where validator_id = $1"#)
            .bind(validator_id.clone())
            .fetch_one(&**pool)
            .await
            .map_err(|e| println!("{}", e))
            .expect("could not fetch network participation");

    let total_attestation_made_by_validator: Count = sqlx::query_as(
        r#"SELECT count(*) as count FROM ATTESTATIONS where validator_id = $1 and attested= true"#,
    )
    .bind(validator_id)
    .fetch_one(&**pool)
    .await
    .map_err(|e| println!("{}", e))
    .expect("could not fetch network participation");

    Ok((total_attestation_made_by_validator.count as f64
        / total_number_of_committess_of_validator.count as f64)
        .to_string())
}

pub async fn calculate_network_participation_of_a_committee(
    epoch_id: String,
    committee_id: String,
    pool: &Extension<PgPool>,
) -> Result<String, Box<dyn Error>> {
    let total_number_of_entries_of_committee: Count = sqlx::query_as(
        r#"SELECT count(*) as count FROM ATTESTATIONS where committee_id = $1 and epoch_id = $2"#,
    )
    .bind(committee_id.parse::<i64>().unwrap())
    .bind(epoch_id.parse::<i64>().unwrap())
    .fetch_one(&**pool)
    .await
    .map_err(|e| println!("{}", e))
    .expect("could not fetch network participation");

    let total_attestation_made_by_committee: Count = sqlx::query_as(
     r#"SELECT count(*) as count FROM ATTESTATIONS where committee_id = $1 and epoch_id = $2 and attested= true"#,
    ).bind(committee_id.parse::<i64>().unwrap())
    .bind(epoch_id.parse::<i64>().unwrap())
    .fetch_one(&**pool).await.map_err(|e| println!("{}", e)).expect(
     "could not fetch network participation");

    Ok((total_attestation_made_by_committee.count as f64
        / total_number_of_entries_of_committee.count as f64)
        .to_string())
}

pub async fn calculate_network_participation_of_an_epoch(
    epoch_id: String,
    pool: &Extension<PgPool>,
) -> Result<String, Box<dyn Error>> {
    let total_number_of_entries_in_epoch: Count =
        sqlx::query_as(r#"SELECT count(*) as count FROM ATTESTATIONS where epoch_id = $1"#)
            .bind(epoch_id.parse::<i64>().unwrap())
            .fetch_one(&**pool)
            .await
            .map_err(|e| println!("{}", e))
            .expect("could not fetch network participation");

    let total_attestation_made_in_epoch: Count = sqlx::query_as(
        r#"SELECT count(*) as count FROM ATTESTATIONS where epoch_id = $1 and attested= true"#,
    )
    .bind(epoch_id.parse::<i64>().unwrap())
    .fetch_one(&**pool)
    .await
    .map_err(|e| println!("{}", e))
    .expect("could not fetch network participation");

    Ok((total_attestation_made_in_epoch.count as f64
        / total_number_of_entries_in_epoch.count as f64)
        .to_string())
}
