//! `forward_ledger_accounts_timezone` reads
//! `acme.ledger.v1.LedgerAccountQueryService/SummarizeLedgerAccounts` and hands what it keeps to the sink of
//! its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ledger::v1 as ledger;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient::new(channel.clone());
    let loaded = source
        .summarize_ledger_accounts(ledger::SummarizeLedgerAccountsRequest {
            context: Some(context.clone()),
            ledger_account_id: "main-forward-ledger-accounts-timezone".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let period = loaded.period.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = LedgerAccountsTimezoneRecord {
        timezone: acme_kit::text::normalize(&period.timezone),
        report_id: loaded.report_id,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn LedgerAccountsTimezoneSink>> = BTreeMap::new();
    sinks.insert(
        "dispatch-order",
        Box::new(DispatchOrderSink {
            client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "gateway-policy-notification",
        Box::new(GatewayPolicyNotificationSink {
            client:
                edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "care-plan-calculation",
        Box::new(CarePlanCalculationSink {
            client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient::new(channel),
        }),
    );
    match sinks.get("care-plan-calculation") {
        Some(sink) => sink.push(record).await,
        None => anyhow::bail!("no sink registered for care-plan-calculation"),
    }
}

/// What `forward_ledger_accounts_timezone` keeps of each SummarizeLedgerAccounts response.
#[derive(Clone, Debug, Default)]
pub struct LedgerAccountsTimezoneRecord {
    pub timezone: String,
    pub report_id: String,
    pub correlation_id: String,
}

/// Where `forward_ledger_accounts_timezone` hands each record.
trait LedgerAccountsTimezoneSink {
    fn push(&self, record: LedgerAccountsTimezoneRecord) -> Delivery;
}

/// Hands records to `acme.dispatch.v1.DispatchOrderService`.
struct DispatchOrderSink {
    client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient<Channel>,
    context: common::RequestContext,
}

impl LedgerAccountsTimezoneSink for DispatchOrderSink {
    fn push(&self, record: LedgerAccountsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(dispatch::CreateDispatchOrderRequest {
                    dispatch_order: Some(dispatch::DispatchOrder {
                        dispatch_order_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.create_dispatch_order(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.GatewayPolicyNotificationService`.
struct GatewayPolicyNotificationSink {
    client: edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient<Channel>,
}

impl LedgerAccountsTimezoneSink for GatewayPolicyNotificationSink {
    fn push(&self, record: LedgerAccountsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .unsubscribe_gateway_policy_updates(edgegateway::UnsubscribeGatewayPolicyUpdatesRequest {
                    details: Some(edgegateway::unsubscribe_gateway_policy_updates_request::Details {
                        template_id: record.timezone,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.careplans.v1.CarePlanCalculationService`.
struct CarePlanCalculationSink {
    client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient<Channel>,
}

impl LedgerAccountsTimezoneSink for CarePlanCalculationSink {
    fn push(&self, record: LedgerAccountsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_care_plan(careplans::SimulateCarePlanRequest {
                    care_plan: Some(careplans::CarePlan {
                        care_plan_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
