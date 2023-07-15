use axum::Extension;
use sqlx::PgPool;
use std::error::Error;
use std::hash::Hash;
use std::{collections::HashMap, time::Instant};

use crate::controller;
use crate::utils::{constants, util_functions};

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
        for slot in ((epoch) * constants::NUMBER_OF_SLOTS_PER_EPOCH)
            ..((epoch + 1) * constants::NUMBER_OF_SLOTS_PER_EPOCH)
        {
            let committee_attestation_bits_mapping: (
                bool,
                Option<HashMap<(i64, String), Vec<bool>>>,
            ) = util_functions::find_committee_attestations_bits_mapping(epoch, slot).await;

            if committee_attestation_bits_mapping.0 {
                match committee_attestation_bits_mapping.1 {
                    Some(val) => {
                        committee_attestation_bits_for_epoch_mapping.extend(val);
                    },
                    None => ()
                }
            } else {
                continue;
            }
        }

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
