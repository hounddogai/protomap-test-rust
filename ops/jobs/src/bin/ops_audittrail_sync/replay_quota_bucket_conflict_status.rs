//! `replay_quota_bucket_conflict_status` reads
//! `acme.ratelimiting.v1.QuotaBucketSyncService/ResolveQuotaBucketConflict` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::lending::v1 as lending;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient::new(channel.clone());
    let found = source
        .resolve_quota_bucket_conflict(ratelimiting::ResolveQuotaBucketConflictRequest {
            context: Some(context.clone()),
            reason: "primary-replay-quota-bucket-conflict-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let quota_bucket = found.quota_bucket.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = QuotaBucketConflictStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        currency_code: acme_kit::text::normalize(&quota_bucket.currency_code),
        quota_bucket_id: quota_bucket.quota_bucket_id,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
    };
    let sinks: Vec<(&str, Box<dyn QuotaBucketConflictStatusWriter>)> = vec![
        (
            "feature-flag-validation",
            Box::new(FeatureFlagValidationSink {
                client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "loan-application-query",
            Box::new(LoanApplicationQuerySink {
                client: lending::loan_application_query_service_client::LoanApplicationQueryServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "feature-flag-validation" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `replay_quota_bucket_conflict_status` keeps of each ResolveQuotaBucketConflict response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketConflictStatusRecord {
    pub status: String,
    pub currency_code: String,
    pub quota_bucket_id: String,
    pub reference_id: String,
}

/// Where `replay_quota_bucket_conflict_status` hands each record.
trait QuotaBucketConflictStatusWriter {
    fn apply(&self, record: QuotaBucketConflictStatusRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FeatureFlagValidationService`.
struct FeatureFlagValidationSink {
    client: featureflags::feature_flag_validation_service_client::FeatureFlagValidationServiceClient<Channel>,
}

impl QuotaBucketConflictStatusWriter for FeatureFlagValidationSink {
    fn apply(&self, record: QuotaBucketConflictStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_feature_flags(featureflags::LintFeatureFlagsRequest {
                    feature_flag: Some(featureflags::FeatureFlag {
                        description: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.status,
                    feature_flag_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.lending.v1.LoanApplicationQueryService`.
struct LoanApplicationQuerySink {
    client: lending::loan_application_query_service_client::LoanApplicationQueryServiceClient<Channel>,
}

impl QuotaBucketConflictStatusWriter for LoanApplicationQuerySink {
    fn apply(&self, record: QuotaBucketConflictStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_loan_applications(lending::SuggestLoanApplicationsRequest {
                    filter: Some(lending::suggest_loan_applications_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    order_by: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
