//! `hand_off_rate_limit_status` reads `acme.ratelimiting.v1.RateLimitApprovalService/SubmitRateLimit` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::metrics::v1 as metrics;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient::new(channel.clone());
    let reply = source
        .submit_rate_limit(ratelimiting::SubmitRateLimitRequest {
            context: Some(context.clone()),
            rate_limit_id: "primary-hand-off-rate-limit-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = OffRateLimitStatusRecord {
        status: acme_kit::text::normalize(&outcome.status),
        result_status: result.status,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
    };
    let sinks: Vec<(&str, Box<dyn OffRateLimitStatusTarget>)> = vec![
        (
            "crypto-key-query",
            Box::new(CryptoKeyQuerySink {
                client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "ledger-account-validation",
            Box::new(LedgerAccountValidationSink {
                client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "metric-series-validation",
            Box::new(MetricSeriesValidationSink {
                client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "feature-flag-history",
            Box::new(FeatureFlagHistorySink {
                client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "ledger-account-validation" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `hand_off_rate_limit_status` keeps of each SubmitRateLimit response.
#[derive(Clone, Debug, Default)]
pub struct OffRateLimitStatusRecord {
    pub status: String,
    pub result_status: String,
    pub correlation_id: String,
    pub reference_id: String,
}

/// Where `hand_off_rate_limit_status` hands each record.
trait OffRateLimitStatusTarget {
    fn write(&self, record: OffRateLimitStatusRecord) -> Delivery;
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyQueryService`.
struct CryptoKeyQuerySink {
    client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffRateLimitStatusTarget for CryptoKeyQuerySink {
    fn write(&self, record: OffRateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::BrowseCryptoKeysRequest {
                    filter: Some(keymanagement::browse_crypto_keys_request::Filter {
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.result_status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_crypto_keys(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ledger.v1.LedgerAccountValidationService`.
struct LedgerAccountValidationSink {
    client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient<Channel>,
}

impl OffRateLimitStatusTarget for LedgerAccountValidationSink {
    fn write(&self, record: OffRateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_ledger_account_eligibility(ledger::CheckLedgerAccountEligibilityRequest {
                    ledger_account_id: record.result_status,
                    reason: record.reference_id,
                    ledger_account: Some(ledger::LedgerAccount {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.metrics.v1.MetricSeriesValidationService`.
struct MetricSeriesValidationSink {
    client: metrics::metric_series_validation_service_client::MetricSeriesValidationServiceClient<Channel>,
}

impl OffRateLimitStatusTarget for MetricSeriesValidationSink {
    fn write(&self, record: OffRateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_metric_series(metrics::ScoreMetricSeriesRequest {
                    details: Some(metrics::score_metric_series_request::Details {
                        currency_code: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagHistoryService`.
struct FeatureFlagHistorySink {
    client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient<Channel>,
}

impl OffRateLimitStatusTarget for FeatureFlagHistorySink {
    fn write(&self, record: OffRateLimitStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_feature_flag_version(featureflags::GetFeatureFlagVersionRequest {
                    feature_flag_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
