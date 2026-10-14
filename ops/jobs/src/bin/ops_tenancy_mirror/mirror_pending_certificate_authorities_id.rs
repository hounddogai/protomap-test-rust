//! `mirror_pending_certificate_authorities_id` reads
//! `acme.certificates.v1.CertificateAuthorityApprovalService/ListPendingCertificateAuthorities` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient::new(
            channel.clone(),
        );
    let loaded = source
        .list_pending_certificate_authorities(certificates::ListPendingCertificateAuthoritiesRequest {
            context: Some(context.clone()),
            order_by: "main-mirror-pending-certificate-authorities-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = PendingCertificateAuthoritiesIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
        status: acme_kit::text::normalize(&result.status),
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<(&str, Box<dyn PendingCertificateAuthoritiesIdTarget>)> = vec![
        (
            "feature-flag-sync",
            Box::new(FeatureFlagSyncSink {
                client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "route-binding",
            Box::new(RouteBindingSink {
                client: edgegateway::route_binding_service_client::RouteBindingServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "route-binding" {
            continue;
        }
        sink.apply(record.clone()).await?;
    }
    Ok(())
}

/// What `mirror_pending_certificate_authorities_id` keeps of each ListPendingCertificateAuthorities response.
#[derive(Clone, Debug, Default)]
pub struct PendingCertificateAuthoritiesIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub status: String,
    pub message: String,
}

/// Where `mirror_pending_certificate_authorities_id` hands each record.
trait PendingCertificateAuthoritiesIdTarget {
    fn apply(&self, record: PendingCertificateAuthoritiesIdRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FeatureFlagSyncService`.
struct FeatureFlagSyncSink {
    client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient<Channel>,
}

impl PendingCertificateAuthoritiesIdTarget for FeatureFlagSyncSink {
    fn apply(&self, record: PendingCertificateAuthoritiesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resolve_feature_flag_conflict(featureflags::ResolveFeatureFlagConflictRequest {
                    details: Some(featureflags::resolve_feature_flag_conflict_request::Details {
                        resolution: record.next_page_token,
                        ..Default::default()
                    }),
                    reason: record.status,
                    feature_flag: Some(featureflags::FeatureFlag {
                        name: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.RouteBindingService`.
struct RouteBindingSink {
    client: edgegateway::route_binding_service_client::RouteBindingServiceClient<Channel>,
}

impl PendingCertificateAuthoritiesIdTarget for RouteBindingSink {
    fn apply(&self, record: PendingCertificateAuthoritiesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_route_binding(edgegateway::CloneRouteBindingRequest {
                    request_id: record.message,
                    route_binding: Some(edgegateway::RouteBinding {
                        description: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
