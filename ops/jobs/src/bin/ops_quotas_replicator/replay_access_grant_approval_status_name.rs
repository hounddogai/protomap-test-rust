//! `replay_access_grant_approval_status_name` reads
//! `acme.secrets.v1.AccessGrantApprovalService/GetAccessGrantApprovalStatus` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(channel.clone());
    let reply = source
        .get_access_grant_approval_status(secrets::GetAccessGrantApprovalStatusRequest {
            context: Some(context.clone()),
            access_grant_id: "nightly-replay-access-grant-approval-status-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let access_grant = reply.access_grant.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = AccessGrantApprovalStatusNameRecord {
        name: access_grant.name,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
        description: access_grant.description,
    };
    let sinks: Vec<(&str, Box<dyn AccessGrantApprovalStatusNameTarget>)> = vec![
        (
            "flag-evaluation-validation",
            Box::new(FlagEvaluationValidationSink {
                client:
                    featureflags::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient::new(
                        channel.clone(),
                    ),
                context: context.clone(),
            }),
        ),
        (
            "crypto-key-calculation",
            Box::new(CryptoKeyCalculationSink {
                client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "anomaly-query",
            Box::new(AnomalyQuerySink {
                client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "flag-evaluation-history",
            Box::new(FlagEvaluationHistorySink {
                client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-evaluation-validation" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `replay_access_grant_approval_status_name` keeps of each GetAccessGrantApprovalStatus response.
#[derive(Clone, Debug, Default)]
pub struct AccessGrantApprovalStatusNameRecord {
    pub name: String,
    pub message: String,
    pub correlation_id: String,
    pub description: String,
}

/// Where `replay_access_grant_approval_status_name` hands each record.
trait AccessGrantApprovalStatusNameTarget {
    fn push(&self, record: AccessGrantApprovalStatusNameRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationValidationService`.
struct FlagEvaluationValidationSink {
    client: featureflags::flag_evaluation_validation_service_client::FlagEvaluationValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl AccessGrantApprovalStatusNameTarget for FlagEvaluationValidationSink {
    fn push(&self, record: AccessGrantApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::ScoreFlagEvaluationRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        correlation_id: record.correlation_id,
                        ..Default::default()
                    }),
                    flag_evaluation_id: record.message,
                    details: Some(featureflags::score_flag_evaluation_request::Details {
                        currency_code: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.score_flag_evaluation(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyCalculationService`.
struct CryptoKeyCalculationSink {
    client: keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient<Channel>,
}

impl AccessGrantApprovalStatusNameTarget for CryptoKeyCalculationSink {
    fn push(&self, record: AccessGrantApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_crypto_key(keymanagement::CalculateCryptoKeyRequest {
                    crypto_key_id: record.name,
                    reason: record.message,
                    details: Some(keymanagement::calculate_crypto_key_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.anomalies.v1.AnomalyQueryService`.
struct AnomalyQuerySink {
    client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient<Channel>,
}

impl AccessGrantApprovalStatusNameTarget for AnomalyQuerySink {
    fn push(&self, record: AccessGrantApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_anomalies(anomalies::SummarizeAnomaliesRequest {
                    reason: record.description,
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

impl AccessGrantApprovalStatusNameTarget for FlagEvaluationHistorySink {
    fn push(&self, record: AccessGrantApprovalStatusNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_flag_evaluation_versions(featureflags::ListFlagEvaluationVersionsRequest {
                    order_by: record.name,
                    filter: Some(featureflags::list_flag_evaluation_versions_request::Filter {
                        region: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
