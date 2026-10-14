//! `route_broadcast_status` reads `acme.broadcasts.v1.BroadcastService/DeleteBroadcast` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone());
    let fetched = source
        .delete_broadcast(broadcasts::DeleteBroadcastRequest {
            context: Some(context.clone()),
            etag: "live-route-broadcast-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = BroadcastStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn BroadcastStatusWriter>> = vec![
        Box::new(FeatureFlagNotificationSink { client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient::new(channel.clone()) }),
        Box::new(CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel.clone()) }),
        Box::new(KeyRingReportingSink { client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient::new(channel.clone()) }),
        Box::new(CsrApprovalSink { client: certificates::csr_approval_service_client::CsrApprovalServiceClient::new(channel) }),
    ];
    sinks[1].write(record).await
}

/// What `route_broadcast_status` keeps of each DeleteBroadcast response.
#[derive(Clone, Debug, Default)]
pub struct BroadcastStatusRecord {
    pub status: String,
    pub correlation_id: String,
}

/// Where `route_broadcast_status` hands each record.
trait BroadcastStatusWriter {
    fn write(&self, record: BroadcastStatusRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FeatureFlagNotificationService`.
struct FeatureFlagNotificationSink {
    client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient<Channel>,
}

impl BroadcastStatusWriter for FeatureFlagNotificationSink {
    fn write(&self, record: BroadcastStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .notify_feature_flag_watchers(featureflags::NotifyFeatureFlagWatchersRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
}

impl BroadcastStatusWriter for CertificateAuthorityValidationSink {
    fn write(&self, record: BroadcastStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_certificate_authorities(certificates::LintCertificateAuthoritiesRequest {
                    details: Some(certificates::lint_certificate_authorities_request::Details {
                        locale: record.correlation_id,
                        ..Default::default()
                    }),
                    certificate_authority: Some(certificates::CertificateAuthority {
                        name: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRingReportingService`.
struct KeyRingReportingSink {
    client: keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient<Channel>,
}

impl BroadcastStatusWriter for KeyRingReportingSink {
    fn write(&self, record: BroadcastStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_key_ring_report(keymanagement::ScheduleKeyRingReportRequest {
                    details: Some(keymanagement::schedule_key_ring_report_request::Details {
                        cron: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CsrApprovalService`.
struct CsrApprovalSink {
    client: certificates::csr_approval_service_client::CsrApprovalServiceClient<Channel>,
}

impl BroadcastStatusWriter for CsrApprovalSink {
    fn write(&self, record: BroadcastStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_csr(certificates::ApproveCsrRequest {
                    reason: record.status,
                    csr: Some(certificates::Csr {
                        subject: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
