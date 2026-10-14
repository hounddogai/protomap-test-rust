//! `carry_quota_bucket_workflow_code` reads
//! `acme.ratelimiting.v1.QuotaBucketWorkflowService/PauseQuotaBucketWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(channel.clone());
    let reply = source
        .pause_quota_bucket_workflow(ratelimiting::PauseQuotaBucketWorkflowRequest {
            context: Some(context.clone()),
            quota_bucket_id: "primary-carry-quota-bucket-workflow-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let quota_bucket = reply.quota_bucket.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = QuotaBucketWorkflowCodeRecord {
        currency_code: quota_bucket.currency_code,
        message: result.message,
        quota_bucket_id: quota_bucket.quota_bucket_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn QuotaBucketWorkflowCodeWriter>> = BTreeMap::new();
    sinks.insert(
        "certificate-authority-query",
        Box::new(CertificateAuthorityQuerySink {
            client:
                certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "route-binding-integration",
        Box::new(RouteBindingIntegrationSink {
            client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("route-binding-integration") else {
        anyhow::bail!("no sink registered for route-binding-integration");
    };
    sink.push(record).await
}

/// What `carry_quota_bucket_workflow_code` keeps of each PauseQuotaBucketWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketWorkflowCodeRecord {
    pub currency_code: String,
    pub message: String,
    pub quota_bucket_id: String,
}

/// Where `carry_quota_bucket_workflow_code` hands each record.
trait QuotaBucketWorkflowCodeWriter {
    fn push(&self, record: QuotaBucketWorkflowCodeRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CertificateAuthorityQueryService`.
struct CertificateAuthorityQuerySink {
    client: certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient<Channel>,
}

impl QuotaBucketWorkflowCodeWriter for CertificateAuthorityQuerySink {
    fn push(&self, record: QuotaBucketWorkflowCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_certificate_authorities(certificates::BrowseCertificateAuthoritiesRequest {
                    filter: Some(certificates::browse_certificate_authorities_request::Filter {
                        query: record.quota_bucket_id,
                        region: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.RouteBindingIntegrationService`.
struct RouteBindingIntegrationSink {
    client: edgegateway::route_binding_integration_service_client::RouteBindingIntegrationServiceClient<Channel>,
}

impl QuotaBucketWorkflowCodeWriter for RouteBindingIntegrationSink {
    fn push(&self, record: QuotaBucketWorkflowCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .replay_route_binding_events(edgegateway::ReplayRouteBindingEventsRequest {
                    details: Some(edgegateway::replay_route_binding_events_request::Details {
                        message_id: record.currency_code,
                        source_system: record.quota_bucket_id,
                        ..Default::default()
                    }),
                    route_binding_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
