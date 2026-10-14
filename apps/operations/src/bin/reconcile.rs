//! Scheduled sync operations, isolated from the domain server's request handling.

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_admin_service_client::AccessPolicyAdminServiceClient;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::environment_sync_service_client::EnvironmentSyncServiceClient;
use acme_proto::acme::configuration::v1::environment_workflow_service_client::EnvironmentWorkflowServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_admin_service_client::FeatureFlagAdminServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_service_client::FeatureFlagServiceClient;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::ratelimiting::v1::quota_bucket_query_service_client::QuotaBucketQueryServiceClient;
use acme_proto::acme::ratelimiting::v1::quota_bucket_sync_service_client::QuotaBucketSyncServiceClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    sweep_access_policy(channel.clone(), context.clone()).await?;
    sweep_environment(channel.clone(), context.clone()).await?;
    sweep_feature_flag(channel.clone(), context.clone()).await?;
    sweep_quota_bucket(channel.clone(), context.clone()).await?;
    Ok(())
}
/// Page through access policies and invoke RepairAccessPolicy for each record.
async fn sweep_access_policy(
    channel: tonic::transport::Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut reader = AccessPolicyServiceClient::new(channel.clone());
    let mut writer = AccessPolicyAdminServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = authorization::ListAccessPoliciesRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.list_access_policies(page).await?.into_inner();
        for item in page.access_policies {
            let command = authorization::RepairAccessPolicyRequest {
                access_policy: Some(item),
                context: Some(context.clone()),
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.repair_access_policy(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through environments and invoke PullEnvironments for each record.
async fn sweep_environment(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = EnvironmentWorkflowServiceClient::new(channel.clone());
    let mut writer = EnvironmentSyncServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = configuration::ListEnvironmentWorkflowsRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.list_environment_workflows(page).await?.into_inner();
        for item in page.environments {
            let command = configuration::PullEnvironmentsRequest {
                context: Some(context.clone()),
                environment: Some(item),
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.pull_environments(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through feature flags and invoke RepairFeatureFlag for each record.
async fn sweep_feature_flag(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = FeatureFlagServiceClient::new(channel.clone());
    let mut writer = FeatureFlagAdminServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = featureflags::ListFeatureFlagsRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.list_feature_flags(page).await?.into_inner();
        for item in page.feature_flags {
            let command = featureflags::RepairFeatureFlagRequest {
                context: Some(context.clone()),
                feature_flag: Some(item),
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.repair_feature_flag(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through quota buckets and invoke PullQuotaBuckets for each record.
async fn sweep_quota_bucket(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = QuotaBucketQueryServiceClient::new(channel.clone());
    let mut writer = QuotaBucketSyncServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = ratelimiting::BrowseQuotaBucketsRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.browse_quota_buckets(page).await?.into_inner();
        for item in page.quota_buckets {
            let command = ratelimiting::PullQuotaBucketsRequest {
                context: Some(context.clone()),
                quota_bucket: Some(item),
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.pull_quota_buckets(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
