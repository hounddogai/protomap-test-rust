//! `hand_off_csr_workflow_message` reads `acme.certificates.v1.CsrWorkflowService/ResumeCsrWorkflow` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_workflow_service_client::CsrWorkflowServiceClient::new(channel.clone());
    let found = source
        .resume_csr_workflow(certificates::ResumeCsrWorkflowRequest {
            context: Some(context.clone()),
            csr_id: "default-hand-off-csr-workflow-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let csr = found.csr.unwrap_or_default();
    let record = OffCsrWorkflowMessageRecord {
        message: outcome.message,
        reference_id: acme_kit::text::normalize(&outcome.reference_id),
        subject: acme_kit::text::normalize(&csr.subject),
        status: outcome.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffCsrWorkflowMessageDestination>> = BTreeMap::new();
    sinks.insert(
        "environment-workflow",
        Box::new(EnvironmentWorkflowSink {
            client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "quota-bucket-workflow",
        Box::new(QuotaBucketWorkflowSink {
            client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
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
    let Some(sink) = sinks.get("key-rotation-lifecycle") else {
        anyhow::bail!("no sink registered for key-rotation-lifecycle");
    };
    sink.forward(record).await
}

/// What `hand_off_csr_workflow_message` keeps of each ResumeCsrWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct OffCsrWorkflowMessageRecord {
    pub message: String,
    pub reference_id: String,
    pub subject: String,
    pub status: String,
}

/// Where `hand_off_csr_workflow_message` hands each record.
trait OffCsrWorkflowMessageDestination {
    fn forward(&self, record: OffCsrWorkflowMessageRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
}

impl OffCsrWorkflowMessageDestination for EnvironmentWorkflowSink {
    fn forward(&self, record: OffCsrWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_environment_workflow(configuration::CancelEnvironmentWorkflowRequest {
                    reason: record.subject,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketWorkflowService`.
struct QuotaBucketWorkflowSink {
    client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffCsrWorkflowMessageDestination for QuotaBucketWorkflowSink {
    fn forward(&self, record: OffCsrWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ListQuotaBucketWorkflowsRequest {
                    filter: Some(ratelimiting::list_quota_bucket_workflows_request::Filter {
                        region: record.status,
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.subject,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_quota_bucket_workflows(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRotationLifecycleService`.
struct KeyRotationLifecycleSink {
    client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient<Channel>,
}

impl OffCsrWorkflowMessageDestination for KeyRotationLifecycleSink {
    fn forward(&self, record: OffCsrWorkflowMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_key_rotation(keymanagement::RenewKeyRotationRequest {
                    details: Some(keymanagement::renew_key_rotation_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
