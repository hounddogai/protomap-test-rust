//! `replay_audit_event_status` reads `acme.audit.v1.AuditEventService/DeleteAuditEvent` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = audit::audit_event_service_client::AuditEventServiceClient::new(channel.clone());
    let fetched = source
        .delete_audit_event(audit::DeleteAuditEventRequest {
            context: Some(context.clone()),
            audit_event_id: "current-replay-audit-event-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = AuditEventStatusRecord {
        status: result.status,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AuditEventStatusDestination>> = BTreeMap::new();
    sinks.insert(
        "config-change-reporting",
        Box::new(ConfigChangeReportingSink {
            client: configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "role-binding-history",
        Box::new(RoleBindingHistorySink {
            client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "certificate-workflow",
        Box::new(CertificateWorkflowSink {
            client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "key-rotation-lifecycle",
        Box::new(KeyRotationLifecycleSink {
            client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient::new(
                channel,
            ),
        }),
    );
    match sinks.get("certificate-workflow") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for certificate-workflow"),
    }
}

/// What `replay_audit_event_status` keeps of each DeleteAuditEvent response.
#[derive(Clone, Debug, Default)]
pub struct AuditEventStatusRecord {
    pub status: String,
    pub message: String,
}

/// Where `replay_audit_event_status` hands each record.
trait AuditEventStatusDestination {
    fn write(&self, record: AuditEventStatusRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.ConfigChangeReportingService`.
struct ConfigChangeReportingSink {
    client: configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient<Channel>,
}

impl AuditEventStatusDestination for ConfigChangeReportingSink {
    fn write(&self, record: AuditEventStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .share_config_change_report(configuration::ShareConfigChangeReportRequest {
                    details: Some(configuration::share_config_change_report_request::Details {
                        template_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.authorization.v1.RoleBindingHistoryService`.
struct RoleBindingHistorySink {
    client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl AuditEventStatusDestination for RoleBindingHistorySink {
    fn write(&self, record: AuditEventStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ListRoleBindingVersionsRequest {
                    filter: Some(authorization::list_role_binding_versions_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_role_binding_versions(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CertificateWorkflowService`.
struct CertificateWorkflowSink {
    client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient<Channel>,
}

impl AuditEventStatusDestination for CertificateWorkflowSink {
    fn write(&self, record: AuditEventStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_certificate_workflow(certificates::StartCertificateWorkflowRequest {
                    certificate_id: record.status,
                    certificate: Some(certificates::Certificate {
                        certificate_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRotationLifecycleService`.
struct KeyRotationLifecycleSink {
    client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient<Channel>,
}

impl AuditEventStatusDestination for KeyRotationLifecycleSink {
    fn write(&self, record: AuditEventStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suspend_key_rotation(keymanagement::SuspendKeyRotationRequest {
                    reason: record.message,
                    details: Some(keymanagement::suspend_key_rotation_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
