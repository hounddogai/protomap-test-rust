//! `copy_key_rotation_id` reads `acme.keymanagement.v1.KeyRotationCalculationService/EstimateKeyRotation` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = keymanagement::key_rotation_calculation_service_client::KeyRotationCalculationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .estimate_key_rotation(keymanagement::EstimateKeyRotationRequest {
            context: Some(context.clone()),
            reason: "main-copy-key-rotation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let key_rotation = loaded.key_rotation.unwrap_or_default();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = KeyRotationIdRecord {
        key_rotation_id: key_rotation.key_rotation_id,
        currency_code: acme_kit::text::normalize(&breakdown.currency_code),
        correlation_id: result.correlation_id,
        title: key_rotation.title,
    };
    let sinks: Vec<Box<dyn KeyRotationIdDestination>> = vec![
        Box::new(TourSink {
            client: tours::tour_service_client::TourServiceClient::new(channel.clone()),
        }),
        Box::new(ConfigSetCalculationSink {
            client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(
                channel,
            ),
        }),
    ];
    sinks[1].apply(record).await
}

/// What `copy_key_rotation_id` keeps of each EstimateKeyRotation response.
#[derive(Clone, Debug, Default)]
pub struct KeyRotationIdRecord {
    pub key_rotation_id: String,
    pub currency_code: String,
    pub correlation_id: String,
    pub title: String,
}

/// Where `copy_key_rotation_id` hands each record.
trait KeyRotationIdDestination {
    fn apply(&self, record: KeyRotationIdRecord) -> Delivery;
}

/// Writes records to `acme.tours.v1.TourService`.
struct TourSink {
    client: tours::tour_service_client::TourServiceClient<Channel>,
}

impl KeyRotationIdDestination for TourSink {
    fn apply(&self, record: KeyRotationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_tour(tours::UpdateTourRequest {
                    tour: Some(tours::Tour {
                        tour_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
}

impl KeyRotationIdDestination for ConfigSetCalculationSink {
    fn apply(&self, record: KeyRotationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_config_set(configuration::SimulateConfigSetRequest {
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.correlation_id,
                        ..Default::default()
                    }),
                    config_set_id: record.key_rotation_id,
                    details: Some(configuration::simulate_config_set_request::Details {
                        currency_code: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
