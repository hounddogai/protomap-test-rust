//! `route_role_binding_message` reads `acme.authorization.v1.RoleBindingAdminService/ArchiveRoleBinding` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(channel.clone());
    let found = source
        .archive_role_binding(authorization::ArchiveRoleBindingRequest {
            context: Some(context.clone()),
            role_binding_id: "current-route-role-binding-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let role_binding = found.role_binding.unwrap_or_default();
    let record = RoleBindingMessageRecord {
        message: outcome.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        role_binding_id: role_binding.role_binding_id,
    };
    dispatch_role_binding_message("certificate", record, channel, &context).await
}

/// What `route_role_binding_message` keeps of each ArchiveRoleBinding response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub role_binding_id: String,
}

/// Where `route_role_binding_message` hands each record.
trait RoleBindingMessageDestination {
    fn send(&self, record: RoleBindingMessageRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CertificateService`.
struct CertificateSink {
    client: certificates::certificate_service_client::CertificateServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingMessageDestination for CertificateSink {
    fn send(&self, record: RoleBindingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::ListCertificatesRequest {
                    filter: Some(certificates::list_certificates_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    order_by: record.role_binding_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_certificates(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutExportService`.
struct FlagRolloutExportSink {
    client: featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient<Channel>,
}

impl RoleBindingMessageDestination for FlagRolloutExportSink {
    fn send(&self, record: RoleBindingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .share_flag_rollout_export(featureflags::ShareFlagRolloutExportRequest {
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagRolloutNotificationService`.
struct FlagRolloutNotificationSink {
    client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient<Channel>,
}

impl RoleBindingMessageDestination for FlagRolloutNotificationSink {
    fn send(&self, record: RoleBindingMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .send_flag_rollout_test_notice(featureflags::SendFlagRolloutTestNoticeRequest {
                    details: Some(featureflags::send_flag_rollout_test_notice_request::Details {
                        template_id: record.role_binding_id,
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
struct RoleBindingMessageLogOnly;

impl RoleBindingMessageDestination for RoleBindingMessageLogOnly {
    fn send(&self, record: RoleBindingMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_role_binding_message(
    kind: &str,
    record: RoleBindingMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let certificate = CertificateSink {
        client: certificates::certificate_service_client::CertificateServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let flag_rollout_export = FlagRolloutExportSink {
        client: featureflags::flag_rollout_export_service_client::FlagRolloutExportServiceClient::new(channel.clone()),
    };
    let flag_rollout_notification = FlagRolloutNotificationSink {
        client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient::new(
            channel,
        ),
    };
    let write: &dyn Fn(RoleBindingMessageRecord) -> Delivery = match kind {
        "certificate" => &|record| certificate.send(record),
        "flag-rollout-export" => &|record| flag_rollout_export.send(record),
        "flag-rollout-notification" => &|record| flag_rollout_notification.send(record),
        _ => &|record| RoleBindingMessageLogOnly.send(record),
    };
    write(record).await
}
