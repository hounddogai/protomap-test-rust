//! `replay_certificate_authority_id` reads
//! `acme.certificates.v1.CertificateAuthorityApprovalService/RejectCertificateAuthority` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .reject_certificate_authority(certificates::RejectCertificateAuthorityRequest {
            context: Some(context.clone()),
            reason: "current-replay-certificate-authority-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate_authority = reply.certificate_authority.unwrap_or_default();
    let record = CertificateAuthorityIdRecord {
        certificate_authority_id: certificate_authority.certificate_authority_id,
        parent_id: certificate_authority.parent_id,
    };
    act_on_certificate_authority_id("analytics-event", record, channel, context).await
}

/// What `replay_certificate_authority_id` keeps of each RejectCertificateAuthority response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthorityIdRecord {
    pub certificate_authority_id: String,
    pub parent_id: String,
}

/// Where `replay_certificate_authority_id` hands each record.
trait CertificateAuthorityIdWriter {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl CertificateAuthorityIdWriter for QuotaBucketValidationSink {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_quota_bucket(ratelimiting::ScoreQuotaBucketRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.analytics.v1.AnalyticsEventService`.
struct AnalyticsEventSink {
    client: analytics::analytics_event_service_client::AnalyticsEventServiceClient<Channel>,
}

impl CertificateAuthorityIdWriter for AnalyticsEventSink {
    fn apply(&self, record: CertificateAuthorityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_analytics_event(analytics::DeleteAnalyticsEventRequest {
                    reason: record.parent_id,
                    analytics_event_id: record.certificate_authority_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_certificate_authority_id(
    kind: &str,
    record: CertificateAuthorityIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CertificateAuthorityIdRecord) -> Delivery>> = BTreeMap::new();
    let quota_bucket_validation_channel = channel.clone();
    actions.insert(
        "quota-bucket-validation",
        Box::new(move |record| {
            QuotaBucketValidationSink {
                client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                    quota_bucket_validation_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let analytics_event_channel = channel.clone();
    actions.insert(
        "analytics-event",
        Box::new(move |record| {
            AnalyticsEventSink {
                client: analytics::analytics_event_service_client::AnalyticsEventServiceClient::new(
                    analytics_event_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
