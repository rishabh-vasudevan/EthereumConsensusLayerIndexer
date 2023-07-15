use axum::Extension;
use reqwest::{self, Client};
use serde_json::Value;
use sqlx::PgPool;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use super::constants;

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
struct Entry {
    id: i32,
    epoch_id: i32,
    slot_id: i32,
    committee_id: i32,
    validator_id: String,
    attested: bool,
}

pub async fn get_request_call_with_param(
    mut url: String,
    parameters: Option<HashMap<String, String>>,
) -> Result<String, Box<dyn Error>> {
    println!(
        "received util function call to make an api call to {:?} with params {:?}",
        url, parameters
    );
    match parameters {
        Some(params) => {
            url += "?";
            for (key, value) in &params {
                url += format!("{}={}&", key, value).as_str();
            }
            url.pop();
        }
        None => (),
    }
    println!("get_request_call_with_param :: the final url is {}", url);
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let res = client.get(url).send().await.unwrap().text().await;

    let final_response = res.unwrap();
    Ok(final_response)
}

pub async fn find_current_epoch() -> i64 {
    println!("find_current_epoch :: request received to find the current epoch number");
    let url = constants::QUICKNODE_BASE_URL.to_string() + "/eth/v1/beacon/headers/head";
    let res = get_request_call_with_param(url, None).await;
    match res {
        Ok(val) => {
            let json_res: Value = serde_json::from_str(&val).unwrap();
            match json_res["data"]["header"]["message"]["slot"]
                .as_str()
                .unwrap()
                .parse::<i64>()
            {
                Ok(slot_num) => slot_num / constants::NUMBER_OF_SLOTS_PER_EPOCH,
                Err(_) => panic!("find_current_epoch :: there was an error in parsing the json"),
            }
        }
        Err(_) => panic!("find_current_epoch :: There was some error in find_current_epoch"),
    }
}

pub async fn find_committee_and_validators_for_epoch(
    epoch: i64,
) -> HashMap<(i64, String), Vec<String>> {
    println!("find_committee_and_validators_for_slot :: request received to find validators in each committee for a slot");
    let url = constants::QUICKNODE_BASE_URL.to_string() + "/eth/v1/beacon/states/head/committees";
    let mut params: HashMap<String, String> = HashMap::new();
    params.insert("epoch".to_string(), epoch.to_string());
    let res = get_request_call_with_param(url, Some(params)).await;
    let mut committee_validators_mapping: HashMap<(i64, String), Vec<String>> = HashMap::new();

    match res {
        Ok(val) => {
            let json_res: Value = serde_json::from_str(&val).unwrap();
            match json_res["data"].as_array() {
                Some(data_array) => {
                    for data in data_array {
                        let slot = data["slot"].as_str().unwrap().parse::<i64>().unwrap();
                        if epoch * constants::NUMBER_OF_SLOTS_PER_EPOCH <= slot
                            && (epoch + 1) * constants::NUMBER_OF_SLOTS_PER_EPOCH > slot
                        {
                            let index = data["index"].as_str().unwrap().to_string();
                            let validators = data["validators"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|val| val.as_str().unwrap().to_string())
                                .collect();
                            committee_validators_mapping.insert((slot, index), validators);
                        }
                    }
                }
                None => {
                    panic!("find_committee_and_validator_for_slot :: none received in data array");
                }
            }
        }
        Err(_) => {
            panic!("find_committee_and_validator_for_slot :: error in parsing the json");
        }
    }
    committee_validators_mapping
}

pub async fn find_committee_attestations_bits_mapping(
    epoch: i64, 
    slot: i64,
) -> (bool, Option<HashMap<(i64, String), Vec<bool>>>) {
    println!("find_committee_attestations_bits_mapping :: request received to find attestations per block");
    let url = constants::QUICKNODE_BASE_URL.to_string()
        + format!("eth/v1/beacon/blocks/{}/attestations", (slot + 1)).as_str();
    let res = get_request_call_with_param(url, None).await;
    let mut committee_attestations_bits_mapping: HashMap<(i64, String), Vec<bool>> = HashMap::new();

    match res {
        Ok(val) => {
            let json_res: Value = serde_json::from_str(&val).unwrap();
            match json_res["data"].as_array() {
                Some(data_array) => {
                    for data in data_array {
                        let aggregation_array =
                            hex_to_boolean_array(data["aggregation_bits"].as_str().unwrap());
                        let committee_index = data["data"]["index"].as_str().unwrap();
                        let committee_slot = data["data"]["slot"].as_str().unwrap().parse::<i64>().unwrap();
                        if committee_slot >= (epoch * constants::NUMBER_OF_SLOTS_PER_EPOCH){
                        committee_attestations_bits_mapping
                            .insert((committee_slot, committee_index.to_string()), aggregation_array);
                        }
                    }
                }
                None => {
                    println!(
                        "find_committee_attestations_bits_mapping :: unable to parse json response"
                    );
                    return (false, None);
                }
            }
        }
        Err(_) => {
            panic!("find_committee_attestations_bits_mapping :: some error in response");
        }
    }

    (true, Some(committee_attestations_bits_mapping))
}

fn little_to_big_endian(hex: &str) -> String {
    let hex = hex.trim_start_matches("0x");
    let reversed_pairs: Vec<_> = hex.chars().rev().collect();
    let pairs = reversed_pairs.chunks(2).rev();
    let big_endian: String = pairs
        .map(|pair| pair.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("");

    let big_endian_hex = format!("0x{}", big_endian);

    big_endian_hex
}

fn reverse_hex_chunks_to_binary(hex: &str) -> String {
    let mut final_binary_string = String::new();
    for digit in hex.chars() {
        final_binary_string += format!(
            "{:04b}",
            u32::from_str_radix(&digit.to_string(), 16).unwrap()
        )
        .chars()
        .rev()
        .collect::<String>()
        .as_str();
    }
    final_binary_string
}

fn hex_to_boolean_array(hex: &str) -> Vec<bool> {
    let big_endian_hex = &little_to_big_endian(hex.clone());
    let binary_string = reverse_hex_chunks_to_binary(&big_endian_hex[2..big_endian_hex.len()]);
    let index = binary_string.rfind('1').unwrap_or(0);
    let substring = &binary_string[..index];
    substring.chars().map(|c| c == '1').collect()
}

pub async fn write_attestation_data_to_postgres(
    committee_validators_mapping: &HashMap<(i64, String), Vec<String>>,
    committee_attestation_bits_mapping: HashMap<(i64, String), Vec<bool>>,
    epoch: i64,
    pool: &Extension<PgPool>,
) -> () {
    let mut insert_many_vector: Vec<(i64, i64, i64, String, bool)> = Vec::new();
    committee_attestation_bits_mapping
        .iter()
        .for_each(|((committee_slot, committee), attestation_bool_arr)| {
            let validators_in_committee = committee_validators_mapping
                .get(&(committee_slot.to_owned(), committee.clone()))
                .unwrap();

            attestation_bool_arr
                .iter()
                .enumerate()
                .for_each(|(validator_index, attested)| {
                    if let Some(validator) = validators_in_committee.get(validator_index) {
                        insert_many_vector.push((
                            epoch,
                            committee_slot.to_owned(),
                            committee.parse::<i64>().unwrap(),
                            validator.clone(),
                            attested.clone(),
                        ));
                    } else {
                        panic!("problem with length matching");
                    }
                });
        });

    let mut epochs: Vec<i64> = Vec::new();
    let mut slots: Vec<i64> = Vec::new();
    let mut committees: Vec<i64> = Vec::new();
    let mut validator_indexes: Vec<String> = Vec::new();
    let mut attestations: Vec<bool> = Vec::new();

    insert_many_vector.into_iter().for_each(|entry| {
        epochs.push(entry.0);
        slots.push(entry.1);
        committees.push(entry.2);
        validator_indexes.push(entry.3);
        attestations.push(entry.4);
    });

    let _insertion_res: Entry = sqlx::query_as(
        r#"INSERT INTO attestations (epoch_id, slot_id, committee_id, validator_id, attested)
        select * from UNNEST ($1, $2, $3, $4, $5) returning id, epoch_id, slot_id, committee_id, validator_id, attested"#,
    )
    .bind(&epochs)
    .bind(&slots)
    .bind(&committees)
    .bind(&validator_indexes)
    .bind(&attestations)
    .fetch_one(&**pool)
    .await.map_err(|e| println!("{}", e)).expect(
        "could not run multiple insert"
    );
}

pub async fn delete_data_in_table(table_name: &str, pool: &Extension<PgPool>) -> () {
    let query = format!("DELETE FROM {}", table_name);
    let _deletion_query = sqlx::query(&query)
        .execute(&**pool)
        .await
        .map_err(|e| println!("{}", e))
        .expect("unable to delete table");
}

#[cfg(test)]
mod tests {

    use super::*;

    // test to check whether get_request_call_with_param function is working
    #[tokio::test]
    async fn get_request_test() {
        let res = get_request_call_with_param("https://www.google.com".into(), None).await;
        match res {
            Ok(_) => (),
            Err(e) => panic!("there was an error : {}", e),
        }
    }

    // test to check whether the get_epoch_function is working and returning the correct return type
    #[tokio::test]
    async fn get_current_epoch_test() {
        let epoch = find_current_epoch().await;
        assert_eq!(type_of(epoch), "i64");
    }

    // test to check whether the attestation_bits are of equal length as of the validator array in a committee
    #[tokio::test]
    async fn match_committee_len_to_aggregation_bits_len() {
        let epoch = 214776 as i64;
        let slot = 6872840 as i64;
        let committee_number = "49";
        let committee_validator_list = find_committee_and_validators_for_epoch(epoch).await;
        let attestation_bits_for_slot = find_committee_attestations_bits_mapping(epoch, slot).await;

        let validators_in_committee = committee_validator_list
            .get(&(slot, committee_number.into()))
            .unwrap();
        let attestations_in_committee_length = match attestation_bits_for_slot.1 {
            Some(val) => val.get(&(slot, committee_number.into())).unwrap().len(),
            None => {
                panic!("test failed got None value");
            }
        };
        assert_eq!(
            validators_in_committee.len(),
            attestations_in_committee_length
        );
    }

    #[tokio::test]
    async fn check_conversion_from_little_endian_hex_to_big_endian_binary() {
        let hex =
            "0xffffffffffffffffffffffffffffffffffffffefffffffffffffffffffffffffffffffffffffffff1f";
        let big_endian_binary: String = hex_to_boolean_array(hex)
            .iter()
            .map(|&value| if value { "1" } else { "0" })
            .collect();

        let correct_binary = "111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111011111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";
        assert_eq!(big_endian_binary, correct_binary);
    }

    fn type_of<T>(_: T) -> &'static str {
        std::any::type_name::<T>()
    }
}
