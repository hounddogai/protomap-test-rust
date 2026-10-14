//! `mirror_ledger_account_decision_code` reads
//! `acme.ledger.v1.LedgerAccountValidationService/ExplainLedgerAccountDecision` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::timekeeping::v1 as timekeeping;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(channel.clone());
    let found = source
        .explain_ledger_account_decision(ledger::ExplainLedgerAccountDecisionRequest {
            context: Some(context.clone()),
            ledger_account_id: "default-mirror-ledger-account-decision-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let ledger_account = found.ledger_account.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = LedgerAccountDecisionCodeRecord {
        currency_code: acme_kit::text::normalize(&ledger_account.currency_code),
        message: outcome.message,
    };
    act_on_ledger_account_decision_code("permission-check-history", record, channel, context).await
}

/// What `mirror_ledger_account_decision_code` keeps of each ExplainLedgerAccountDecision response.
#[derive(Clone, Debug, Default)]
pub struct LedgerAccountDecisionCodeRecord {
    pub currency_code: String,
    pub message: String,
}

/// Where `mirror_ledger_account_decision_code` hands each record.
trait LedgerAccountDecisionCodeTarget {
    fn write(&self, record: LedgerAccountDecisionCodeRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.PermissionCheckHistoryService`.
struct PermissionCheckHistorySink {
    client: authorization::permission_check_history_service_client::PermissionCheckHistoryServiceClient<Channel>,
}

impl LedgerAccountDecisionCodeTarget for PermissionCheckHistorySink {
    fn write(&self, record: LedgerAccountDecisionCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .annotate_permission_check_version(authorization::AnnotatePermissionCheckVersionRequest {
                    permission_check: Some(authorization::PermissionCheck {
                        permission_check_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.QuotaBucketCalculationService`.
struct QuotaBucketCalculationSink {
    client: ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LedgerAccountDecisionCodeTarget for QuotaBucketCalculationSink {
    fn write(&self, record: LedgerAccountDecisionCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::EstimateQuotaBucketRequest {
                    quota_bucket_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.estimate_quota_bucket(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.timekeeping.v1.TimesheetQueryService`.
struct TimesheetQuerySink {
    client: timekeeping::timesheet_query_service_client::TimesheetQueryServiceClient<Channel>,
}

impl LedgerAccountDecisionCodeTarget for TimesheetQuerySink {
    fn write(&self, record: LedgerAccountDecisionCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_timesheets(timekeeping::BrowseTimesheetsRequest {
                    filter: Some(timekeeping::browse_timesheets_request::Filter {
                        owner_id: record.currency_code,
                        query: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_ledger_account_decision_code(
    kind: &str,
    record: LedgerAccountDecisionCodeRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(LedgerAccountDecisionCodeRecord) -> Delivery>> = BTreeMap::new();
    let permission_check_history_channel = channel.clone();
    actions.insert(
        "permission-check-history",
        Box::new(move |record| {
            PermissionCheckHistorySink {
                client:
                    authorization::permission_check_history_service_client::PermissionCheckHistoryServiceClient::new(
                        permission_check_history_channel.clone(),
                    ),
            }
            .write(record)
        }),
    );
    let quota_bucket_calculation_channel = channel.clone();
    let quota_bucket_calculation_context = context.clone();
    actions.insert(
        "quota-bucket-calculation",
        Box::new(move |record| {
            QuotaBucketCalculationSink {
                client: ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient::new(
                    quota_bucket_calculation_channel.clone(),
                ),
                context: quota_bucket_calculation_context.clone(),
            }
            .write(record)
        }),
    );
    let timesheet_query_channel = channel.clone();
    actions.insert(
        "timesheet-query",
        Box::new(move |record| {
            TimesheetQuerySink {
                client: timekeeping::timesheet_query_service_client::TimesheetQueryServiceClient::new(
                    timesheet_query_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
