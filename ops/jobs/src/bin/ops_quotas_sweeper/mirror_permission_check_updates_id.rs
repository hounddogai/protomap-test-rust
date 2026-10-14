//! `mirror_permission_check_updates_id` reads
//! `acme.authorization.v1.PermissionCheckNotificationService/UnsubscribePermissionCheckUpdates` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .unsubscribe_permission_check_updates(authorization::UnsubscribePermissionCheckUpdatesRequest {
            context: Some(context.clone()),
            permission_check_id: "main-mirror-permission-check-updates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let permission_check = reply.permission_check.unwrap_or_default();
    let record = PermissionCheckUpdatesIdRecord {
        correlation_id: result.correlation_id,
        permission_check_id: acme_kit::text::normalize(&permission_check.permission_check_id),
        permission_check_correlation_id: permission_check.correlation_id,
    };
    deliver_permission_check_updates_id("certificate", record, channel, &context).await
}

/// What `mirror_permission_check_updates_id` keeps of each UnsubscribePermissionCheckUpdates response.
#[derive(Clone, Debug, Default)]
pub struct PermissionCheckUpdatesIdRecord {
    pub correlation_id: String,
    pub permission_check_id: String,
    pub permission_check_correlation_id: String,
}

/// Where `mirror_permission_check_updates_id` hands each record.
trait PermissionCheckUpdatesIdTarget {
    fn forward(&self, record: PermissionCheckUpdatesIdRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FlagRolloutValidationService`.
struct FlagRolloutValidationSink {
    client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient<Channel>,
}

impl PermissionCheckUpdatesIdTarget for FlagRolloutValidationSink {
    fn forward(&self, record: PermissionCheckUpdatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_flag_rollout(featureflags::NormalizeFlagRolloutRequest {
                    reason: record.permission_check_id,
                    flag_rollout: Some(featureflags::FlagRollout {
                        title: record.correlation_id,
                        flag_rollout_id: record.permission_check_correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
}

impl PermissionCheckUpdatesIdTarget for SponsorQuerySink {
    fn forward(&self, record: PermissionCheckUpdatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_sponsor(sponsorships::LookupSponsorRequest {
                    sponsor_id: record.permission_check_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.certificates.v1.CertificateService`.
struct CertificateSink {
    client: certificates::certificate_service_client::CertificateServiceClient<Channel>,
}

impl PermissionCheckUpdatesIdTarget for CertificateSink {
    fn forward(&self, record: PermissionCheckUpdatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_certificates(certificates::SearchCertificatesRequest {
                    filter: Some(certificates::search_certificates_request::Filter {
                        owner_id: record.permission_check_id,
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.permission_check_correlation_id,
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
struct PermissionCheckUpdatesIdLogOnly;

impl PermissionCheckUpdatesIdTarget for PermissionCheckUpdatesIdLogOnly {
    fn forward(&self, record: PermissionCheckUpdatesIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_permission_check_updates_id(
    kind: &str,
    record: PermissionCheckUpdatesIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn PermissionCheckUpdatesIdTarget> = match kind {
        "flag-rollout-validation" => Box::new(FlagRolloutValidationSink {
            client: featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "sponsor-query" => Box::new(SponsorQuerySink {
            client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel.clone()),
        }),
        "certificate" => Box::new(CertificateSink {
            client: certificates::certificate_service_client::CertificateServiceClient::new(channel),
        }),
        _ => Box::new(PermissionCheckUpdatesIdLogOnly),
    };
    sink.forward(record).await
}
