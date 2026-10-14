//! `sync_seat_map_name` reads `acme.seating.v1.SeatMapCalculationService/QuoteSeatMap` and hands what it
//! keeps to `acme.keymanagement.v1.CryptoKeyCalculationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel.clone());
    let reply = source
        .quote_seat_map(seating::QuoteSeatMapRequest {
            context: Some(context.clone()),
            seat_map_id: "nightly-sync-seat-map-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let seat_map = reply.seat_map.unwrap_or_default();
    let record = SeatMapNameRecord {
        name: seat_map.name,
        parent_id: seat_map.parent_id,
    };
    CryptoKeyCalculationSink {
        client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(channel),
    }
    .write(record)
    .await
}

/// What `sync_seat_map_name` keeps of each QuoteSeatMap response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapNameRecord {
    pub name: String,
    pub parent_id: String,
}

/// Where `sync_seat_map_name` hands each record.
trait SeatMapNameWriter {
    fn write(&self, record: SeatMapNameRecord) -> Delivery;
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl SeatMapNameWriter for CryptoKeyCalculationSink {
    fn write(&self, record: SeatMapNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_crypto_key(keymanagement::CalculateCryptoKeyRequest {
                    reason: record.parent_id,
                    crypto_key: Some(keymanagement::CryptoKey {
                        crypto_key_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
