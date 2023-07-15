use axum::Extension;
use futures;
use sqlx::PgPool;
use std::future::Future;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashMap, time::Instant};

use crate::controller;
use crate::utils::{constants, util_functions};

async fn join_parallel<T: Send + 'static>(
    futs: impl IntoIterator<Item = impl Future<Output = T> + Send + 'static>,
) -> Vec<T> {
    let tasks: Vec<_> = futs.into_iter().map(tokio::spawn).collect();
    // unwrap the Result because it is introduced by tokio::spawn()
    // and isn't something our caller can handle
    futures::future::join_all(tasks)
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect()
}

pub async fn run_indexer_impl(pool: Extension<PgPool>) -> Result<(), Box<dyn Error>> {
    //find the current epoch head go from (ep_head-6, ep_head -1)

    let current_epoch = util_functions::find_current_epoch().await;
    println!(
        "run_indexer_impl :: the current epoch number is : {:?}",
        current_epoch
    );
    util_functions::delete_data_in_table(constants::TABLE_NAME, &pool).await;
    let start_time = Instant::now();

    for epoch in (current_epoch - constants::NUMBER_OF_EPOCHS)..current_epoch {
        let committee_validators_mapping: HashMap<(i64, String), Vec<String>> =
            util_functions::find_committee_and_validators_for_epoch(epoch).await;
        let mut committee_attestation_bits_for_epoch_mapping: HashMap<(i64, String), Vec<bool>> =
            HashMap::new();

        let range = ((epoch as u64) * constants::NUMBER_OF_SLOTS_PER_EPOCH as u64)
            ..((epoch as u64) * constants::NUMBER_OF_SLOTS_PER_EPOCH as u64 + 16);
        let stream: Vec<(bool, Option<HashMap<(i64, String), Vec<bool>>>)> =
            join_parallel(range.into_iter().map(|slot| async move {
                util_functions::find_committee_attestations_bits_mapping(epoch.clone(), slot as i64)
                    .await
            }))
            .await;

        for attestation_in_slot in &stream {
            if let Some(val) = &attestation_in_slot.1 {
                committee_attestation_bits_for_epoch_mapping.extend(val.to_owned());
            }
        }

        sleep(Duration::from_secs(1));

        let range2 = ((epoch as u64) * constants::NUMBER_OF_SLOTS_PER_EPOCH as u64 + 16)
            ..((epoch as u64 + 1) * constants::NUMBER_OF_SLOTS_PER_EPOCH as u64);
        let stream2: Vec<(bool, Option<HashMap<(i64, String), Vec<bool>>>)> =
            join_parallel(range2.into_iter().map(|slot| async move {
                util_functions::find_committee_attestations_bits_mapping(epoch.clone(), slot as i64)
                    .await
            }))
            .await;

        for attestation_in_slot in &stream2 {
            if let Some(val) = &attestation_in_slot.1 {
                committee_attestation_bits_for_epoch_mapping.extend(val.to_owned());
            }
        }

        sleep(Duration::from_secs(1));
        if committee_attestation_bits_for_epoch_mapping.len() > 0 {
            util_functions::write_attestation_data_to_postgres(
                &committee_validators_mapping,
                committee_attestation_bits_for_epoch_mapping,
                epoch,
                &pool,
            )
            .await;
        }
    }

    println!("run_indexer_impl :: it took {:?} ", start_time.elapsed());

    Ok(())
}

#[derive(sqlx::FromRow)]
struct Epochs {
    epoch_id: i32,
}

#[derive(sqlx::FromRow)]
struct Slots {
    slot_id: i32,
}

#[derive(sqlx::FromRow)]
struct Validators {
    validator_id: String,
}

pub async fn get_data_about_current_state(
    pool: &Extension<PgPool>,
) -> Result<controller::indexer::CurrentUniqueData, Box<dyn Error>> {
    let epochs: Vec<Epochs> =
        sqlx::query_as(r#"select distinct epoch_id as epoch_id from attestations"#)
            .fetch_all(&**pool)
            .await?;

    let slots: Vec<Slots> =
        sqlx::query_as(r#"select distinct slot_id as slot_id from attestations"#)
            .fetch_all(&**pool)
            .await?;

    let validators: Vec<Validators> =
        sqlx::query_as(r#"select distinct validator_id as validator_id from attestations"#)
            .fetch_all(&**pool)
            .await?;

    Ok(controller::indexer::CurrentUniqueData {
        epochs: epochs
            .into_iter()
            .map(|item| item.epoch_id.to_string())
            .collect(),
        slots: slots
            .into_iter()
            .map(|item| item.slot_id.to_string())
            .collect(),
        validators: validators
            .into_iter()
            .map(|item| item.validator_id)
            .collect(),
    })
}
