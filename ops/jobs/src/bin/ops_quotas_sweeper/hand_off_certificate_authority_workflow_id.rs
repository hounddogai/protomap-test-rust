//! `hand_off_certificate_authority_workflow_id` reads
//! `acme.certificates.v1.CertificateAuthorityWorkflowService/AdvanceCertificateAuthorityWorkflow` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_workflow_service_client::CertificateAuthorityWorkflowServiceClient::new(
            channel.clone(),
        );
    let found = source
        .advance_certificate_authority_workflow(certificates::AdvanceCertificateAuthorityWorkflowRequest {
            context: Some(context.clone()),
            certificate_authority_id: "current-hand-off-certificate-authority-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = OffCertificateAuthorityWorkflowIdRecord {
        correlation_id: result.correlation_id,
        status: outcome.status,
        message: acme_kit::text::normalize(&result.message),
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
    };
    dispatch_off_certificate_authority_workflow_id("broadcast", record, channel, &context).await
}

/// What `hand_off_certificate_authority_workflow_id` keeps of each AdvanceCertificateAuthorityWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct OffCertificateAuthorityWorkflowIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub message: String,
    pub reference_id: String,
}

/// Where `hand_off_certificate_authority_workflow_id` hands each record.
trait OffCertificateAuthorityWorkflowIdSink {
    fn apply(&self, record: OffCertificateAuthorityWorkflowIdRecord) -> Delivery;
}

/// Sends records to `acme.broadcasts.v1.BroadcastService`.
struct BroadcastSink {
    client: broadcasts::broadcast_service_client::BroadcastServiceClient<Channel>,
}

impl OffCertificateAuthorityWorkflowIdSink for BroadcastSink {
    fn apply(&self, record: OffCertificateAuthorityWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_broadcast(broadcasts::CreateBroadcastRequest {
                    request_id: record.reference_id,
                    broadcast: Some(broadcasts::Broadcast {
                        broadcast_id: record.correlation_id,
                        title: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyApprovalService`.
struct CryptoKeyApprovalSink {
    client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient<Channel>,
}

impl OffCertificateAuthorityWorkflowIdSink for CryptoKeyApprovalSink {
    fn apply(&self, record: OffCertificateAuthorityWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_pending_crypto_keys(keymanagement::ListPendingCryptoKeysRequest {
                    order_by: record.message,
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    filter: Some(keymanagement::list_pending_crypto_keys_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigSetValidationService`.
struct ConfigSetValidationSink {
    client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient<Channel>,
}

impl OffCertificateAuthorityWorkflowIdSink for ConfigSetValidationSink {
    fn apply(&self, record: OffCertificateAuthorityWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_config_set(configuration::NormalizeConfigSetRequest {
                    config_set: Some(configuration::ConfigSet {
                        config_set_id: record.reference_id,
                        name: record.message,
                        ..Default::default()
                    }),
                    details: Some(configuration::normalize_config_set_request::Details {
                        locale: record.correlation_id,
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
struct OffCertificateAuthorityWorkflowIdLogOnly;

impl OffCertificateAuthorityWorkflowIdSink for OffCertificateAuthorityWorkflowIdLogOnly {
    fn apply(&self, record: OffCertificateAuthorityWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_off_certificate_authority_workflow_id(
    kind: &str,
    record: OffCertificateAuthorityWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let broadcast = BroadcastSink {
        client: broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone()),
    };
    let crypto_key_approval = CryptoKeyApprovalSink {
        client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(channel.clone()),
    };
    let config_set_validation = ConfigSetValidationSink {
        client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient::new(channel),
    };
    let write: &dyn Fn(OffCertificateAuthorityWorkflowIdRecord) -> Delivery = match kind {
        "broadcast" => &|record| broadcast.apply(record),
        "crypto-key-approval" => &|record| crypto_key_approval.apply(record),
        "config-set-validation" => &|record| config_set_validation.apply(record),
        _ => &|record| OffCertificateAuthorityWorkflowIdLogOnly.apply(record),
    };
    write(record).await
}
