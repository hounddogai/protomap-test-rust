//! `backfill_role_binding_export_notes` reads
//! `acme.authorization.v1.RoleBindingExportService/RetryRoleBindingExport` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::itsm::v1 as itsm;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_export_service_client::RoleBindingExportServiceClient::new(channel.clone());
    let loaded = source
        .retry_role_binding_export(authorization::RetryRoleBindingExportRequest {
            context: Some(context.clone()),
            reason: "primary-backfill-role-binding-export-notes".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let role_binding = loaded.role_binding.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = RoleBindingExportNotesRecord {
        notes: role_binding.notes,
        correlation_id: result.correlation_id,
        reference_id: outcome.reference_id,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn RoleBindingExportNotesDestination>)> = vec![
        ("key-rotation-approval", Box::new(KeyRotationApprovalSink { client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient::new(channel.clone()) })),
        ("incident", Box::new(IncidentSink { client: itsm::incident_service_client::IncidentServiceClient::new(channel.clone()), context: context.clone() })),
        ("config-set-approval", Box::new(ConfigSetApprovalSink { client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient::new(channel.clone()), context: context.clone() })),
        ("certificate-authority-validation", Box::new(CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel), context: context.clone() })),
    ];
    for (kind, sink) in &sinks {
        if *kind != "key-rotation-approval" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `backfill_role_binding_export_notes` keeps of each RetryRoleBindingExport response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingExportNotesRecord {
    pub notes: String,
    pub correlation_id: String,
    pub reference_id: String,
    pub status: String,
}

/// Where `backfill_role_binding_export_notes` hands each record.
trait RoleBindingExportNotesDestination {
    fn deliver(&self, record: RoleBindingExportNotesRecord) -> Delivery;
}

/// Sends records to `acme.keymanagement.v1.KeyRotationApprovalService`.
struct KeyRotationApprovalSink {
    client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient<Channel>,
}

impl RoleBindingExportNotesDestination for KeyRotationApprovalSink {
    fn deliver(&self, record: RoleBindingExportNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recall_key_rotation(keymanagement::RecallKeyRotationRequest {
                    details: Some(keymanagement::recall_key_rotation_request::Details {
                        approver_email: record.notes,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.itsm.v1.IncidentService`.
struct IncidentSink {
    client: itsm::incident_service_client::IncidentServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingExportNotesDestination for IncidentSink {
    fn deliver(&self, record: RoleBindingExportNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(itsm::CloneIncidentRequest {
                    request_id: record.status,
                    incident: Some(itsm::Incident {
                        incident_id: record.notes,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_incident(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigSetApprovalService`.
struct ConfigSetApprovalSink {
    client: configuration::config_set_approval_service_client::ConfigSetApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingExportNotesDestination for ConfigSetApprovalSink {
    fn deliver(&self, record: RoleBindingExportNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::CommentConfigSetApprovalRequest {
                    details: Some(configuration::comment_config_set_approval_request::Details {
                        approver_email: record.reference_id,
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

/// Hands records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
    context: common::RequestContext,
}

impl RoleBindingExportNotesDestination for CertificateAuthorityValidationSink {
    fn deliver(&self, record: RoleBindingExportNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::LintCertificateAuthoritiesRequest {
                    certificate_authority: Some(certificates::CertificateAuthority {
                        parent_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_certificate_authorities(request).await?;
            Ok(())
        })
    }
}
