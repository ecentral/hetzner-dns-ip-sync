use serde::{Deserialize, Serialize};
use reqwest::{Method, RequestBuilder};
use local_ip_address::local_ip;
use std::env;

use crate::api::{Records, Zone};

mod api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let my_local_ip = local_ip().unwrap();
    let zone = env::var("HETZNER_ZONE").unwrap();
    let domain = env::var("HETZNER_DOMAIN").unwrap();
    create_update_record(zone.as_str(),domain.as_str(), my_local_ip.to_string().as_str(), "A").await;
    let response = get_all_records_by_name(zone.as_str()).await;
    println!("{:#?}", response.records);
    Ok(())
}

async fn get_zone_by_name(name: &str) -> Zone {
    let zones = api::zones::get_zones(Option::from(name)).await.unwrap();
    zones.zones.into_iter().next().unwrap()
}

async fn get_all_records_by_name(zone_name: &str) -> Records {
    let zone = get_zone_by_name(zone_name).await;
    api::records::get_all_records(zone.id)
}

async fn delete_records_by_name(zone: &str, record_name: &str) {
    let data = get_all_records_by_name(zone)
        .await
        .records
        .into_iter()
        .filter(|record| record.name == record_name);
    for record in data {
        api::records::delete_record(record.id.as_str()).await;
    }
}

async fn create_update_record(zone_name: &str, record_name: &str, value: &str, record_type: &str) {
    let count = get_all_records_by_name(zone_name)
        .await
        .records
        .into_iter()
        .filter(|record| record.name == record_name).count();
    if count > 1 {
        println!("delete");
        delete_records_by_name(zone_name, record_name).await;
    }
    if count == 1 {
        println!("update");
        let mut data = get_all_records_by_name(zone_name).await.records.into_iter()
            .filter(|record| record.name == record_name);
        let record = data.next().unwrap();
        let zone = get_zone_by_name(zone_name).await;
        api::records::update_record(record.id.as_str(), record_name, record_type, value, zone.id).await;
        return;
    }
    println!("add");
    let zone = get_zone_by_name(zone_name).await;
    api::records::create_record(record_name, record_type, value, zone.id).await;
}