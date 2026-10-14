//! `settle_secret_versions_id` reads `acme.secrets.v1.SecretVersionValidationService/LintSecretVersions` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::secret_version_validation_service_client::SecretVersionValidationServiceClient::new(channel.clone());
    let reply = source
        .lint_secret_versions(secrets::LintSecretVersionsRequest {
            context: Some(context.clone()),
            secret_version_id: "primary-settle-secret-versions-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let secret_version = reply.secret_version.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = SecretVersionsIdRecord {
        reference_id: outcome.reference_id,
        correlation_id: acme_kit::text::normalize(&secret_version.correlation_id),
        message: result.message,
        secret_version_id: secret_version.secret_version_id,
    };
    deliver_secret_versions_id("ledger-account-query", record, channel, &context).await
}

/// What `settle_secret_versions_id` keeps of each LintSecretVersions response.
#[derive(Clone, Debug, Default)]
pub struct SecretVersionsIdRecord {
    pub reference_id: String,
    pub correlation_id: String,
    pub message: String,
    pub secret_version_id: String,
}

/// Where `settle_secret_versions_id` hands each record.
trait SecretVersionsIdTarget {
    fn send(&self, record: SecretVersionsIdRecord) -> Delivery;
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketQueryService`.
struct QuotaBucketQuerySink {
    client: ratelimiting::quota_bucket_query_service_client::QuotaBucketQueryServiceClient<Channel>,
}

impl SecretVersionsIdTarget for QuotaBucketQuerySink {
    fn send(&self, record: SecretVersionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_quota_buckets(ratelimiting::SummarizeQuotaBucketsRequest {
                    quota_bucket_id: record.correlation_id,
                    details: Some(ratelimiting::summarize_quota_buckets_request::Details {
                        format: record.secret_version_id,
                        ..Default::default()
                    }),
                    reason: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ledger.v1.LedgerAccountQueryService`.
struct LedgerAccountQuerySink {
    client: ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient<Channel>,
}

impl SecretVersionsIdTarget for LedgerAccountQuerySink {
    fn send(&self, record: SecretVersionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_ledger_account(ledger::LookupLedgerAccountRequest {
                    ledger_account_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct SecretVersionsIdLogOnly;

impl SecretVersionsIdTarget for SecretVersionsIdLogOnly {
    fn send(&self, record: SecretVersionsIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.reference_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_secret_versions_id(
    kind: &str,
    record: SecretVersionsIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SecretVersionsIdTarget> = match kind {
        "quota-bucket-query" => Box::new(QuotaBucketQuerySink {
            client: ratelimiting::quota_bucket_query_service_client::QuotaBucketQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        "ledger-account-query" => Box::new(LedgerAccountQuerySink {
            client: ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient::new(channel),
        }),
        _ => Box::new(SecretVersionsIdLogOnly),
    };
    sink.send(record).await
}
