//! `backfill_get_datasets_status` reads `acme.datasets.v1.DatasetService/BatchGetDatasets` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::datasets::v1 as datasets;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = datasets::dataset_service_client::DatasetServiceClient::new(channel.clone());
    let reply = source
        .batch_get_datasets(datasets::BatchGetDatasetsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = GetDatasetsStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn GetDatasetsStatusDestination>> = vec![
        Box::new(SponsorCalculationSink {
            client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(SecretApprovalSink {
            client: secrets::secret_approval_service_client::SecretApprovalServiceClient::new(channel.clone()),
        }),
        Box::new(KeyRingValidationSink {
            client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(channel),
        }),
    ];
    for sink in &sinks {
        sink.forward(record.clone()).await?;
    }
    Ok(())
}

/// What `backfill_get_datasets_status` keeps of each BatchGetDatasets response.
#[derive(Clone, Debug, Default)]
pub struct GetDatasetsStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `backfill_get_datasets_status` hands each record.
trait GetDatasetsStatusDestination {
    fn forward(&self, record: GetDatasetsStatusRecord) -> Delivery;
}

/// Writes records to `acme.sponsorships.v1.SponsorCalculationService`.
struct SponsorCalculationSink {
    client: sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetDatasetsStatusDestination for SponsorCalculationSink {
    fn forward(&self, record: GetDatasetsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::GetSponsorRatesRequest {
                    sponsor_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_sponsor_rates(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.SecretApprovalService`.
struct SecretApprovalSink {
    client: secrets::secret_approval_service_client::SecretApprovalServiceClient<Channel>,
}

impl GetDatasetsStatusDestination for SecretApprovalSink {
    fn forward(&self, record: GetDatasetsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_secret(secrets::ApproveSecretRequest {
                    secret: Some(secrets::Secret {
                        secret_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRingValidationService`.
struct KeyRingValidationSink {
    client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient<Channel>,
}

impl GetDatasetsStatusDestination for KeyRingValidationSink {
    fn forward(&self, record: GetDatasetsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_key_ring_rules(keymanagement::GetKeyRingRulesRequest {
                    key_ring_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
