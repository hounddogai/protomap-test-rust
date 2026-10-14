//! `replay_config_set_workflow_id` reads
//! `acme.configuration.v1.ConfigSetWorkflowService/PauseConfigSetWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(channel.clone());
    let found = source
        .pause_config_set_workflow(configuration::PauseConfigSetWorkflowRequest {
            context: Some(context.clone()),
            config_set_id: "default-replay-config-set-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_set = found.config_set.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = ConfigSetWorkflowIdRecord {
        parent_id: config_set.parent_id,
        config_set_id: acme_kit::text::normalize(&config_set.config_set_id),
        name: config_set.name,
        status: acme_kit::text::normalize(&result.status),
    };
    let sinks: Vec<Box<dyn ConfigSetWorkflowIdWriter>> = vec![
        Box::new(ForumSink {
            client: community::forum_service_client::ForumServiceClient::new(channel.clone()),
        }),
        Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(QuotaBucketReportingSink {
            client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    ];
    for sink in &sinks {
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `replay_config_set_workflow_id` keeps of each PauseConfigSetWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetWorkflowIdRecord {
    pub parent_id: String,
    pub config_set_id: String,
    pub name: String,
    pub status: String,
}

/// Where `replay_config_set_workflow_id` hands each record.
trait ConfigSetWorkflowIdWriter {
    fn write(&self, record: ConfigSetWorkflowIdRecord) -> Delivery;
}

/// Writes records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl ConfigSetWorkflowIdWriter for ForumSink {
    fn write(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_forum(community::CloneForumRequest {
                    forum: Some(community::Forum {
                        name: record.parent_id,
                        forum_id: record.status,
                        parent_id: record.config_set_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetWorkflowIdWriter for AccessGrantValidationSink {
    fn write(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::LintAccessGrantsRequest {
                    access_grant: Some(secrets::AccessGrant {
                        access_grant_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_access_grants(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetWorkflowIdWriter for QuotaBucketReportingSink {
    fn write(&self, record: ConfigSetWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::DeleteQuotaBucketReportRequest {
                    reason: record.config_set_id,
                    quota_bucket_id: record.status,
                    etag: record.name,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_quota_bucket_report(request).await?;
            Ok(())
        })
    }
}
