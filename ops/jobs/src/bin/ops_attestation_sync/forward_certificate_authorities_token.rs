//! `forward_certificate_authorities_token` reads
//! `acme.certificates.v1.CertificateAuthorityQueryService/ResolveCertificateAuthorities` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .resolve_certificate_authorities(certificates::ResolveCertificateAuthoritiesRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = reply.page_info.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = CertificateAuthoritiesTokenRecord {
        next_page_token: page_info.next_page_token,
        status: result.status,
        message: result.message,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn CertificateAuthoritiesTokenWriter>> = BTreeMap::new();
    sinks.insert(
        "environment-lifecycle",
        Box::new(EnvironmentLifecycleSink {
            client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "environment-workflow",
        Box::new(EnvironmentWorkflowSink {
            client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("environment-workflow") else {
        anyhow::bail!("no sink registered for environment-workflow");
    };
    sink.write(record).await
}

/// What `forward_certificate_authorities_token` keeps of each ResolveCertificateAuthorities response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthoritiesTokenRecord {
    pub next_page_token: String,
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `forward_certificate_authorities_token` hands each record.
trait CertificateAuthoritiesTokenWriter {
    fn write(&self, record: CertificateAuthoritiesTokenRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.EnvironmentLifecycleService`.
struct EnvironmentLifecycleSink {
    client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient<Channel>,
}

impl CertificateAuthoritiesTokenWriter for EnvironmentLifecycleSink {
    fn write(&self, record: CertificateAuthoritiesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_environment(configuration::RenewEnvironmentRequest {
                    details: Some(configuration::renew_environment_request::Details {
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

/// Hands records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
}

impl CertificateAuthoritiesTokenWriter for EnvironmentWorkflowSink {
    fn write(&self, record: CertificateAuthoritiesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .signal_environment_workflow(configuration::SignalEnvironmentWorkflowRequest {
                    details: Some(configuration::signal_environment_workflow_request::Details {
                        initiated_by: record.message,
                        ..Default::default()
                    }),
                    reason: record.status,
                    environment_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
