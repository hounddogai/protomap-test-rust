//! `forward_fixed_assets_status` reads `acme.assets.v1.FixedAssetQueryService/ResolveFixedAssets` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient::new(channel.clone());
    let found = source
        .resolve_fixed_assets(assets::ResolveFixedAssetsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = FixedAssetsStatusRecord {
        status: result.status,
        next_page_token: page_info.next_page_token,
        message: result.message,
    };
    deliver_fixed_assets_status("access-policy-query", record, channel, &context).await
}

/// What `forward_fixed_assets_status` keeps of each ResolveFixedAssets response.
#[derive(Clone, Debug, Default)]
pub struct FixedAssetsStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `forward_fixed_assets_status` hands each record.
trait FixedAssetsStatusTarget {
    fn apply(&self, record: FixedAssetsStatusRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
    context: common::RequestContext,
}

impl FixedAssetsStatusTarget for QuotaBucketReportingSink {
    fn apply(&self, record: FixedAssetsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ShareQuotaBucketReportRequest {
                    details: Some(ratelimiting::share_quota_bucket_report_request::Details {
                        template_id: record.message,
                        channel: record.status,
                        ..Default::default()
                    }),
                    reason: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.share_quota_bucket_report(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ledger.v1.LedgerAccountService`.
struct LedgerAccountSink {
    client: ledger::ledger_account_service_client::LedgerAccountServiceClient<Channel>,
}

impl FixedAssetsStatusTarget for LedgerAccountSink {
    fn apply(&self, record: FixedAssetsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_ledger_account(ledger::CloneLedgerAccountRequest {
                    ledger_account: Some(ledger::LedgerAccount {
                        ledger_account_id: record.next_page_token,
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    request_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.AccessPolicyQueryService`.
struct AccessPolicyQuerySink {
    client: authorization::access_policy_query_service_client::AccessPolicyQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FixedAssetsStatusTarget for AccessPolicyQuerySink {
    fn apply(&self, record: FixedAssetsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::SummarizeAccessPoliciesRequest {
                    reason: record.message,
                    access_policy_id: record.status,
                    details: Some(authorization::summarize_access_policies_request::Details {
                        cron: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_access_policies(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct FixedAssetsStatusLogOnly;

impl FixedAssetsStatusTarget for FixedAssetsStatusLogOnly {
    fn apply(&self, record: FixedAssetsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_fixed_assets_status(
    kind: &str,
    record: FixedAssetsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn FixedAssetsStatusTarget> = match kind {
        "quota-bucket-reporting" => Box::new(QuotaBucketReportingSink {
            client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "ledger-account" => Box::new(LedgerAccountSink {
            client: ledger::ledger_account_service_client::LedgerAccountServiceClient::new(channel.clone()),
        }),
        "access-policy-query" => Box::new(AccessPolicyQuerySink {
            client: authorization::access_policy_query_service_client::AccessPolicyQueryServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(FixedAssetsStatusLogOnly),
    };
    sink.apply(record).await
}
