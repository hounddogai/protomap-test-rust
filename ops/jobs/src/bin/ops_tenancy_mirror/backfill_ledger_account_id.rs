//! `backfill_ledger_account_id` reads `acme.ledger.v1.LedgerAccountValidationService/PreviewLedgerAccount`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ledger::v1 as ledger;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(channel.clone());
    let reply = source
        .preview_ledger_account(ledger::PreviewLedgerAccountRequest {
            context: Some(context.clone()),
            ledger_account_id: "current-backfill-ledger-account-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let ledger_account = reply.ledger_account.unwrap_or_default();
    let record = LedgerAccountIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        currency_code: ledger_account.currency_code,
    };
    deliver_ledger_account_id("care-plan-query", record, channel, &context).await
}

/// What `backfill_ledger_account_id` keeps of each PreviewLedgerAccount response.
#[derive(Clone, Debug, Default)]
pub struct LedgerAccountIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub currency_code: String,
}

/// Where `backfill_ledger_account_id` hands each record.
trait LedgerAccountIdSink {
    fn write(&self, record: LedgerAccountIdRecord) -> Delivery;
}

/// Writes records to `acme.careplans.v1.CarePlanQueryService`.
struct CarePlanQuerySink {
    client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl LedgerAccountIdSink for CarePlanQuerySink {
    fn write(&self, record: LedgerAccountIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(careplans::BrowseCarePlansRequest {
                    filter: Some(careplans::browse_care_plans_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_care_plans(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.RoleBindingValidationService`.
struct RoleBindingValidationSink {
    client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient<Channel>,
}

impl LedgerAccountIdSink for RoleBindingValidationSink {
    fn write(&self, record: LedgerAccountIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_role_binding(authorization::ScoreRoleBindingRequest {
                    role_binding: Some(authorization::RoleBinding {
                        notes: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ledger.v1.LedgerAccountValidationService`.
struct LedgerAccountValidationSink {
    client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient<Channel>,
}

impl LedgerAccountIdSink for LedgerAccountValidationSink {
    fn write(&self, record: LedgerAccountIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_ledger_account_rules(ledger::GetLedgerAccountRulesRequest {
                    ledger_account_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct LedgerAccountIdLogOnly;

impl LedgerAccountIdSink for LedgerAccountIdLogOnly {
    fn write(&self, record: LedgerAccountIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_ledger_account_id(
    kind: &str,
    record: LedgerAccountIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn LedgerAccountIdSink> = match kind {
        "care-plan-query" => Box::new(CarePlanQuerySink {
            client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "role-binding-validation" => Box::new(RoleBindingValidationSink {
            client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "ledger-account-validation" => Box::new(LedgerAccountValidationSink {
            client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(
                channel,
            ),
        }),
        _ => Box::new(LedgerAccountIdLogOnly),
    };
    sink.write(record).await
}
