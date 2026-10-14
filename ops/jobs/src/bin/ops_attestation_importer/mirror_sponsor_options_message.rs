//! `mirror_sponsor_options_message` reads
//! `acme.sponsorships.v1.SponsorCalculationService/CompareSponsorOptions` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        sponsorships::sponsor_calculation_service_client::SponsorCalculationServiceClient::new(channel.clone());
    let fetched = source
        .compare_sponsor_options(sponsorships::CompareSponsorOptionsRequest {
            context: Some(context.clone()),
            sponsor_id: "primary-mirror-sponsor-options-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let breakdown = fetched.breakdown.unwrap_or_default();
    let sponsor = fetched.sponsor.unwrap_or_default();
    let record = SponsorOptionsMessageRecord {
        message: result.message,
        currency_code: breakdown.currency_code,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        sponsor_id: sponsor.sponsor_id,
    };
    deliver_sponsor_options_message("key-rotation-workflow", record, channel, &context).await
}

/// What `mirror_sponsor_options_message` keeps of each CompareSponsorOptions response.
#[derive(Clone, Debug, Default)]
pub struct SponsorOptionsMessageRecord {
    pub message: String,
    pub currency_code: String,
    pub correlation_id: String,
    pub sponsor_id: String,
}

/// Where `mirror_sponsor_options_message` hands each record.
trait SponsorOptionsMessageDestination {
    fn write(&self, record: SponsorOptionsMessageRecord) -> Delivery;
}

/// Writes records to `acme.anomalies.v1.AnomalyQueryService`.
struct AnomalyQuerySink {
    client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorOptionsMessageDestination for AnomalyQuerySink {
    fn write(&self, record: SponsorOptionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(anomalies::SummarizeAnomaliesRequest {
                    details: Some(anomalies::summarize_anomalies_request::Details {
                        format: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    anomaly_id: record.sponsor_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_anomalies(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl SponsorOptionsMessageDestination for KeyRotationWorkflowSink {
    fn write(&self, record: SponsorOptionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_key_rotation_workflow(keymanagement::RetryKeyRotationWorkflowRequest {
                    key_rotation_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SponsorOptionsMessageDestination for ThrottleCalculationSink {
    fn write(&self, record: SponsorOptionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CalculateThrottleRequest {
                    throttle: Some(ratelimiting::Throttle {
                        correlation_id: record.correlation_id,
                        throttle_id: record.sponsor_id,
                        ..Default::default()
                    }),
                    throttle_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_throttle(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct SponsorOptionsMessageLogOnly;

impl SponsorOptionsMessageDestination for SponsorOptionsMessageLogOnly {
    fn write(&self, record: SponsorOptionsMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_sponsor_options_message(
    kind: &str,
    record: SponsorOptionsMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SponsorOptionsMessageDestination> = match kind {
        "anomaly-query" => Box::new(AnomalyQuerySink {
            client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "key-rotation-workflow" => Box::new(KeyRotationWorkflowSink {
            client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        "throttle-calculation" => Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(SponsorOptionsMessageLogOnly),
    };
    sink.write(record).await
}
