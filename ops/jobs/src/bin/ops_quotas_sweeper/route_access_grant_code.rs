//! `route_access_grant_code` reads `acme.secrets.v1.AccessGrantCalculationService/QuoteAccessGrant` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::access_grant_calculation_service_client::AccessGrantCalculationServiceClient::new(channel.clone());
    let reply = source
        .quote_access_grant(secrets::QuoteAccessGrantRequest {
            context: Some(context.clone()),
            access_grant_id: "nightly-route-access-grant-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = reply.breakdown.unwrap_or_default();
    let access_grant = reply.access_grant.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = AccessGrantCodeRecord {
        currency_code: breakdown.currency_code,
        name: acme_kit::text::normalize(&access_grant.name),
        correlation_id: result.correlation_id,
    };
    deliver_access_grant_code("environment-calculation", record, channel, &context).await
}

/// What `route_access_grant_code` keeps of each QuoteAccessGrant response.
#[derive(Clone, Debug, Default)]
pub struct AccessGrantCodeRecord {
    pub currency_code: String,
    pub name: String,
    pub correlation_id: String,
}

/// Where `route_access_grant_code` hands each record.
trait AccessGrantCodeTarget {
    fn send(&self, record: AccessGrantCodeRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRotationValidationService`.
struct KeyRotationValidationSink {
    client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient<Channel>,
}

impl AccessGrantCodeTarget for KeyRotationValidationSink {
    fn send(&self, record: AccessGrantCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_key_rotation_eligibility(keymanagement::CheckKeyRotationEligibilityRequest {
                    key_rotation_id: record.correlation_id,
                    key_rotation: Some(keymanagement::KeyRotation {
                        key_rotation_id: record.currency_code,
                        title: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationHistoryService`.
struct FlagEvaluationHistorySink {
    client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient<Channel>,
}

impl AccessGrantCodeTarget for FlagEvaluationHistorySink {
    fn send(&self, record: AccessGrantCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .record_flag_evaluation_version(featureflags::RecordFlagEvaluationVersionRequest {
                    details: Some(featureflags::record_flag_evaluation_version_request::Details {
                        author_email: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.EnvironmentCalculationService`.
struct EnvironmentCalculationSink {
    client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient<Channel>,
}

impl AccessGrantCodeTarget for EnvironmentCalculationSink {
    fn send(&self, record: AccessGrantCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_environment(configuration::SimulateEnvironmentRequest {
                    reason: record.name,
                    environment: Some(configuration::Environment {
                        name: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AccessGrantCodeLogOnly;

impl AccessGrantCodeTarget for AccessGrantCodeLogOnly {
    fn send(&self, record: AccessGrantCodeRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.currency_code);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_access_grant_code(
    kind: &str,
    record: AccessGrantCodeRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AccessGrantCodeTarget> = match kind {
        "key-rotation-validation" => Box::new(KeyRotationValidationSink {
            client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "flag-evaluation-history" => Box::new(FlagEvaluationHistorySink {
            client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient::new(
                channel.clone(),
            ),
        }),
        "environment-calculation" => Box::new(EnvironmentCalculationSink {
            client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient::new(
                channel,
            ),
        }),
        _ => Box::new(AccessGrantCodeLogOnly),
    };
    sink.send(record).await
}
