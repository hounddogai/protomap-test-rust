//! `backfill_crypto_key_code` reads `acme.keymanagement.v1.CryptoKeyCalculationService/SimulateCryptoKey` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::payroll::v1 as payroll;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_calculation_service_client::CryptoKeyCalculationServiceClient::new(channel.clone());
    let found = source
        .simulate_crypto_key(keymanagement::SimulateCryptoKeyRequest {
            context: Some(context.clone()),
            reason: "default-backfill-crypto-key-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let crypto_key = found.crypto_key.unwrap_or_default();
    let breakdown = found.breakdown.unwrap_or_default();
    let record = CryptoKeyCodeRecord {
        currency_code: acme_kit::text::normalize(&crypto_key.currency_code),
        crypto_key_id: crypto_key.crypto_key_id,
        breakdown_currency_code: acme_kit::text::normalize(&breakdown.currency_code),
    };
    act_on_crypto_key_code("pay-run", record, channel, context).await
}

/// What `backfill_crypto_key_code` keeps of each SimulateCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyCodeRecord {
    pub currency_code: String,
    pub crypto_key_id: String,
    pub breakdown_currency_code: String,
}

/// Where `backfill_crypto_key_code` hands each record.
trait CryptoKeyCodeTarget {
    fn deliver(&self, record: CryptoKeyCodeRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.ConfigSetApprovalService`.
struct ConfigSetApprovalSink {
    client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyCodeTarget for ConfigSetApprovalSink {
    fn deliver(&self, record: CryptoKeyCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::CommentConfigSetApprovalRequest {
                    details: Some(configuration::comment_config_set_approval_request::Details {
                        approver_email: record.crypto_key_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.comment_config_set_approval(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
}

impl CryptoKeyCodeTarget for CryptoKeyWorkflowSink {
    fn deliver(&self, record: CryptoKeyCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_crypto_key_workflow(keymanagement::StartCryptoKeyWorkflowRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        crypto_key_id: record.crypto_key_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.payroll.v1.PayRunService`.
struct PayRunSink {
    client: payroll::pay_run_service_client::PayRunServiceClient<Channel>,
}

impl CryptoKeyCodeTarget for PayRunSink {
    fn deliver(&self, record: CryptoKeyCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_pay_run(payroll::CreatePayRunRequest {
                    pay_run: Some(payroll::PayRun {
                        pay_run_id: record.currency_code,
                        number: record.crypto_key_id,
                        ..Default::default()
                    }),
                    request_id: record.breakdown_currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_crypto_key_code(
    kind: &str,
    record: CryptoKeyCodeRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CryptoKeyCodeRecord) -> Delivery>> = BTreeMap::new();
    let config_set_approval_channel = channel.clone();
    let config_set_approval_context = context.clone();
    actions.insert(
        "config-set-approval",
        Box::new(move |record| {
            ConfigSetApprovalSink {
                client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(
                    config_set_approval_channel.clone(),
                ),
                context: config_set_approval_context.clone(),
            }
            .deliver(record)
        }),
    );
    let crypto_key_workflow_channel = channel.clone();
    actions.insert(
        "crypto-key-workflow",
        Box::new(move |record| {
            CryptoKeyWorkflowSink {
                client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(
                    crypto_key_workflow_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let pay_run_channel = channel.clone();
    actions.insert(
        "pay-run",
        Box::new(move |record| {
            PayRunSink {
                client: payroll::pay_run_service_client::PayRunServiceClient::new(pay_run_channel.clone()),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
