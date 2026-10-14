//! `relay_api_key_workflow_id` reads `acme.edgegateway.v1.ApiKeyWorkflowService/StartApiKeyWorkflow` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient::new(channel.clone());
    let fetched = source
        .start_api_key_workflow(edgegateway::StartApiKeyWorkflowRequest {
            context: Some(context.clone()),
            reason: "default-relay-api-key-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let api_key = fetched.api_key.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = ApiKeyWorkflowIdRecord {
        api_key_id: acme_kit::text::normalize(&api_key.api_key_id),
        status: outcome.status,
    };
    deliver_api_key_workflow_id("feature-flag-admin", record, channel, &context).await
}

/// What `relay_api_key_workflow_id` keeps of each StartApiKeyWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ApiKeyWorkflowIdRecord {
    pub api_key_id: String,
    pub status: String,
}

/// Where `relay_api_key_workflow_id` hands each record.
trait ApiKeyWorkflowIdDestination {
    fn deliver(&self, record: ApiKeyWorkflowIdRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FeatureFlagAdminService`.
struct FeatureFlagAdminSink {
    client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient<Channel>,
}

impl ApiKeyWorkflowIdDestination for FeatureFlagAdminSink {
    fn deliver(&self, record: ApiKeyWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .purge_feature_flag(featureflags::PurgeFeatureFlagRequest {
                    feature_flag_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FeatureFlagSyncService`.
struct FeatureFlagSyncSink {
    client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl ApiKeyWorkflowIdDestination for FeatureFlagSyncSink {
    fn deliver(&self, record: ApiKeyWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::ListFeatureFlagConflictsRequest {
                    order_by: record.api_key_id,
                    filter: Some(featureflags::list_feature_flag_conflicts_request::Filter {
                        region: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_feature_flag_conflicts(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ApiKeyWorkflowIdLogOnly;

impl ApiKeyWorkflowIdDestination for ApiKeyWorkflowIdLogOnly {
    fn deliver(&self, record: ApiKeyWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.api_key_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_api_key_workflow_id(
    kind: &str,
    record: ApiKeyWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn ApiKeyWorkflowIdDestination> = match kind {
        "feature-flag-admin" => Box::new(FeatureFlagAdminSink {
            client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient::new(
                channel.clone(),
            ),
        }),
        "feature-flag-sync" => Box::new(FeatureFlagSyncSink {
            client: featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(ApiKeyWorkflowIdLogOnly),
    };
    sink.deliver(record).await
}
