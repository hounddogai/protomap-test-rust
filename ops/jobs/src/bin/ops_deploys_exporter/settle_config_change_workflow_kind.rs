//! `settle_config_change_workflow_kind` reads
//! `acme.configuration.v1.ConfigChangeWorkflowService/StartConfigChangeWorkflow` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(channel.clone());
    let reply = source
        .start_config_change_workflow(configuration::StartConfigChangeWorkflowRequest {
            context: Some(context.clone()),
            config_change_id: "live-settle-config-change-workflow-kind".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_change = reply.config_change.unwrap_or_default();
    let record = ConfigChangeWorkflowKindRecord {
        kind: acme_kit::text::normalize(&config_change.kind),
        correlation_id: config_change.correlation_id,
    };
    dispatch_config_change_workflow_kind("csr-calculation", record, channel, &context).await
}

/// What `settle_config_change_workflow_kind` keeps of each StartConfigChangeWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeWorkflowKindRecord {
    pub kind: String,
    pub correlation_id: String,
}

/// Where `settle_config_change_workflow_kind` hands each record.
trait ConfigChangeWorkflowKindWriter {
    fn push(&self, record: ConfigChangeWorkflowKindRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CsrCalculationService`.
struct CsrCalculationSink {
    client: certificates::csr_calculation_service_client::CsrCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangeWorkflowKindWriter for CsrCalculationSink {
    fn push(&self, record: ConfigChangeWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::EstimateCsrRequest {
                    csr_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.estimate_csr(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
}

impl ConfigChangeWorkflowKindWriter for CryptoKeyWorkflowSink {
    fn push(&self, record: ConfigChangeWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .signal_crypto_key_workflow(keymanagement::SignalCryptoKeyWorkflowRequest {
                    crypto_key_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.PermissionCheckHistoryService`.
struct PermissionCheckHistorySink {
    client: authorization::permission_check_history_service_client::PermissionCheckHistoryServiceClient<Channel>,
}

impl ConfigChangeWorkflowKindWriter for PermissionCheckHistorySink {
    fn push(&self, record: ConfigChangeWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .record_permission_check_version(authorization::RecordPermissionCheckVersionRequest {
                    permission_check: Some(authorization::PermissionCheck {
                        kind: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl ConfigChangeWorkflowKindWriter for FanSessionSink {
    fn push(&self, record: ConfigChangeWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_fan_session(fanportal::GetFanSessionRequest {
                    fan_session_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ConfigChangeWorkflowKindLogOnly;

impl ConfigChangeWorkflowKindWriter for ConfigChangeWorkflowKindLogOnly {
    fn push(&self, record: ConfigChangeWorkflowKindRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.kind);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_config_change_workflow_kind(
    kind: &str,
    record: ConfigChangeWorkflowKindRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let csr_calculation = CsrCalculationSink {
        client: certificates::csr_calculation_service_client::CsrCalculationServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let crypto_key_workflow = CryptoKeyWorkflowSink {
        client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel.clone()),
    };
    let permission_check_history = PermissionCheckHistorySink {
        client: authorization::permission_check_history_service_client::PermissionCheckHistoryServiceClient::new(
            channel.clone(),
        ),
    };
    let fan_session = FanSessionSink {
        client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel),
    };
    let write: &dyn Fn(ConfigChangeWorkflowKindRecord) -> Delivery = match kind {
        "csr-calculation" => &|record| csr_calculation.push(record),
        "crypto-key-workflow" => &|record| crypto_key_workflow.push(record),
        "permission-check-history" => &|record| permission_check_history.push(record),
        "fan-session" => &|record| fan_session.push(record),
        _ => &|record| ConfigChangeWorkflowKindLogOnly.push(record),
    };
    write(record).await
}
