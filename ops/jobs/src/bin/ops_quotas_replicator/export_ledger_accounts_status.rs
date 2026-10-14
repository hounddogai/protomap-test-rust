//! `export_ledger_accounts_status` reads `acme.ledger.v1.LedgerAccountQueryService/SuggestLedgerAccounts` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient::new(channel.clone());
    let reply = source
        .suggest_ledger_accounts(ledger::SuggestLedgerAccountsRequest {
            context: Some(context.clone()),
            order_by: "main-export-ledger-accounts-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = LedgerAccountsStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
    };
    dispatch_ledger_accounts_status("config-change-calculation", record, channel, &context).await
}

/// What `export_ledger_accounts_status` keeps of each SuggestLedgerAccounts response.
#[derive(Clone, Debug, Default)]
pub struct LedgerAccountsStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `export_ledger_accounts_status` hands each record.
trait LedgerAccountsStatusWriter {
    fn deliver(&self, record: LedgerAccountsStatusRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.ConfigChangeCalculationService`.
struct ConfigChangeCalculationSink {
    client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient<Channel>,
}

impl LedgerAccountsStatusWriter for ConfigChangeCalculationSink {
    fn deliver(&self, record: LedgerAccountsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_config_change(configuration::EstimateConfigChangeRequest {
                    reason: record.next_page_token,
                    details: Some(configuration::estimate_config_change_request::Details {
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

/// Hands records to `acme.secrets.v1.AccessGrantApprovalService`.
struct AccessGrantApprovalSink {
    client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient<Channel>,
}

impl LedgerAccountsStatusWriter for AccessGrantApprovalSink {
    fn deliver(&self, record: LedgerAccountsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_access_grant(secrets::ApproveAccessGrantRequest {
                    details: Some(secrets::approve_access_grant_request::Details {
                        comment: record.correlation_id,
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
struct LedgerAccountsStatusLogOnly;

impl LedgerAccountsStatusWriter for LedgerAccountsStatusLogOnly {
    fn deliver(&self, record: LedgerAccountsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_ledger_accounts_status(
    kind: &str,
    record: LedgerAccountsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let config_change_calculation = ConfigChangeCalculationSink {
        client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let access_grant_approval = AccessGrantApprovalSink {
        client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(channel),
    };
    let write: &dyn Fn(LedgerAccountsStatusRecord) -> Delivery = match kind {
        "config-change-calculation" => &|record| config_change_calculation.deliver(record),
        "access-grant-approval" => &|record| access_grant_approval.deliver(record),
        _ => &|record| LedgerAccountsStatusLogOnly.deliver(record),
    };
    write(record).await
}
