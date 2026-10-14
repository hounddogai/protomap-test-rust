//! `push_api_key_workflow_code` reads `acme.edgegateway.v1.ApiKeyWorkflowService/SignalApiKeyWorkflow` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::analytics::v1 as analytics;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .signal_api_key_workflow(edgegateway::SignalApiKeyWorkflowRequest {
            context: Some(context.clone()),
            reason: "main-push-api-key-workflow-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let api_key = fetched.api_key.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = ApiKeyWorkflowCodeRecord {
        currency_code: api_key.currency_code,
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
        reference_id: outcome.reference_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ApiKeyWorkflowCodeTarget>> = BTreeMap::new();
    sinks.insert(
        "environment-approval",
        Box::new(EnvironmentApprovalSink {
            client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "analytics-event",
        Box::new(AnalyticsEventSink {
            client: analytics::analytics_event_service_client::AnalyticsEventServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    match sinks.get("analytics-event") {
        Some(sink) => sink.send(record).await,
        None => anyhow::bail!("no sink registered for analytics-event"),
    }
}

/// What `push_api_key_workflow_code` keeps of each SignalApiKeyWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeyWorkflowCodeRecord {
    pub currency_code: String,
    pub status: String,
    pub message: String,
    pub reference_id: String,
}

/// Where `push_api_key_workflow_code` hands each record.
trait ApiKeyWorkflowCodeTarget {
    fn send(&self, record: ApiKeyWorkflowCodeRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
}

impl ApiKeyWorkflowCodeTarget for EnvironmentApprovalSink {
    fn send(&self, record: ApiKeyWorkflowCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recall_environment(configuration::RecallEnvironmentRequest {
                    details: Some(configuration::recall_environment_request::Details {
                        approver_email: record.reference_id,
                        ..Default::default()
                    }),
                    environment: Some(configuration::Environment {
                        environment_id: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.analytics.v1.AnalyticsEventService`.
struct AnalyticsEventSink {
    client: analytics::analytics_event_service_client::AnalyticsEventServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeyWorkflowCodeTarget for AnalyticsEventSink {
    fn send(&self, record: ApiKeyWorkflowCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(analytics::DeleteAnalyticsEventRequest {
                    etag: record.reference_id,
                    analytics_event_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_analytics_event(request).await?;
            Ok(())
        })
    }
}
