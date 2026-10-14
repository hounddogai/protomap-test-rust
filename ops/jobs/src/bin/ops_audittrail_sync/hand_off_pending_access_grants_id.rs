//! `hand_off_pending_access_grants_id` reads
//! `acme.secrets.v1.AccessGrantApprovalService/ListPendingAccessGrants` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(channel.clone());
    let reply = source
        .list_pending_access_grants(secrets::ListPendingAccessGrantsRequest {
            context: Some(context.clone()),
            order_by: "primary-hand-off-pending-access-grants-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = OffPendingAccessGrantsIdRecord {
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
    };
    let sinks: Vec<(&str, Box<dyn OffPendingAccessGrantsIdTarget>)> = vec![
        (
            "crypto-key-query",
            Box::new(CryptoKeyQuerySink {
                client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "admission-calculation",
            Box::new(AdmissionCalculationSink {
                client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "ledger-account-query",
            Box::new(LedgerAccountQuerySink {
                client: ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "admission-calculation" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `hand_off_pending_access_grants_id` keeps of each ListPendingAccessGrants response.
#[derive(Clone, Debug, Default)]
pub struct OffPendingAccessGrantsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `hand_off_pending_access_grants_id` hands each record.
trait OffPendingAccessGrantsIdTarget {
    fn push(&self, record: OffPendingAccessGrantsIdRecord) -> Delivery;
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyQueryService`.
struct CryptoKeyQuerySink {
    client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient<Channel>,
}

impl OffPendingAccessGrantsIdTarget for CryptoKeyQuerySink {
    fn push(&self, record: OffPendingAccessGrantsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_crypto_keys(keymanagement::BrowseCryptoKeysRequest {
                    filter: Some(keymanagement::browse_crypto_keys_request::Filter {
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    order_by: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.admissions.v1.AdmissionCalculationService`.
struct AdmissionCalculationSink {
    client: admissions::admission_calculation_service_client::AdmissionCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffPendingAccessGrantsIdTarget for AdmissionCalculationSink {
    fn push(&self, record: OffPendingAccessGrantsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::SimulateAdmissionRequest {
                    reason: record.correlation_id,
                    admission: Some(admissions::Admission {
                        source: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_admission(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ledger.v1.LedgerAccountQueryService`.
struct LedgerAccountQuerySink {
    client: ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient<Channel>,
}

impl OffPendingAccessGrantsIdTarget for LedgerAccountQuerySink {
    fn push(&self, record: OffPendingAccessGrantsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_ledger_accounts(ledger::SuggestLedgerAccountsRequest {
                    filter: Some(ledger::suggest_ledger_accounts_request::Filter {
                        owner_id: record.correlation_id,
                        query: record.status,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
