//! `route_config_sets_id` reads `acme.configuration.v1.ConfigSetValidationService/LintConfigSets` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ledger::v1 as ledger;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_validation_service_client::ConfigSetValidationServiceClient::new(channel.clone());
    let loaded = source
        .lint_config_sets(configuration::LintConfigSetsRequest {
            context: Some(context.clone()),
            reason: "live-route-config-sets-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = ConfigSetsIdRecord {
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&outcome.status),
    };
    dispatch_config_sets_id("ledger-account-validation", record, channel, &context).await
}

/// What `route_config_sets_id` keeps of each LintConfigSets response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetsIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `route_config_sets_id` hands each record.
trait ConfigSetsIdWriter {
    fn deliver(&self, record: ConfigSetsIdRecord) -> Delivery;
}

/// Hands records to `acme.ledger.v1.LedgerAccountValidationService`.
struct LedgerAccountValidationSink {
    client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient<Channel>,
}

impl ConfigSetsIdWriter for LedgerAccountValidationSink {
    fn deliver(&self, record: ConfigSetsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_ledger_account(ledger::PreviewLedgerAccountRequest {
                    ledger_account: Some(ledger::LedgerAccount {
                        ledger_account_id: record.status,
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

/// Hands records to `acme.featureflags.v1.FlagRolloutExportService`.
struct FlagRolloutExportSink {
    client: featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient<Channel>,
}

impl ConfigSetsIdWriter for FlagRolloutExportSink {
    fn deliver(&self, record: ConfigSetsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_flag_rollout_export(featureflags::ScheduleFlagRolloutExportRequest {
                    details: Some(featureflags::schedule_flag_rollout_export_request::Details {
                        cron: record.correlation_id,
                        format: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateApprovalService`.
struct CertificateApprovalSink {
    client: certificates::certificate_approval_service_client::CertificateApprovalServiceClient<Channel>,
}

impl ConfigSetsIdWriter for CertificateApprovalSink {
    fn deliver(&self, record: ConfigSetsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_certificate(certificates::ApproveCertificateRequest {
                    certificate: Some(certificates::Certificate {
                        number: record.status,
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

/// Only logs, for kinds no service takes yet.
struct ConfigSetsIdLogOnly;

impl ConfigSetsIdWriter for ConfigSetsIdLogOnly {
    fn deliver(&self, record: ConfigSetsIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_config_sets_id(
    kind: &str,
    record: ConfigSetsIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let ledger_account_validation = LedgerAccountValidationSink {
        client: ledger::ledger_account_validation_service_client::LedgerAccountValidationServiceClient::new(
            channel.clone(),
        ),
    };
    let flag_rollout_export = FlagRolloutExportSink {
        client: featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient::new(channel.clone()),
    };
    let certificate_approval = CertificateApprovalSink {
        client: certificates::certificate_approval_service_client::CertificateApprovalServiceClient::new(channel),
    };
    let write: &dyn Fn(ConfigSetsIdRecord) -> Delivery = match kind {
        "ledger-account-validation" => &|record| ledger_account_validation.deliver(record),
        "flag-rollout-export" => &|record| flag_rollout_export.deliver(record),
        "certificate-approval" => &|record| certificate_approval.deliver(record),
        _ => &|record| ConfigSetsIdLogOnly.deliver(record),
    };
    write(record).await
}
