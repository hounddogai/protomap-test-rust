//! Scheduled lifecycle operations, isolated from the domain server's request handling.

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_admin_service_client::AccessPolicyAdminServiceClient;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::csr_lifecycle_service_client::CsrLifecycleServiceClient;
use acme_proto::acme::certificates::v1::csr_workflow_service_client::CsrWorkflowServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient;
use acme_proto::acme::configuration::v1::config_change_query_service_client::ConfigChangeQueryServiceClient;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::featureflags::v1::feature_flag_admin_service_client::FeatureFlagAdminServiceClient;
use acme_proto::acme::featureflags::v1::feature_flag_service_client::FeatureFlagServiceClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    sweep_access_policy(channel.clone(), context.clone()).await?;
    sweep_csr(channel.clone(), context.clone()).await?;
    sweep_config_change(channel.clone(), context.clone()).await?;
    sweep_feature_flag(channel.clone(), context.clone()).await?;
    Ok(())
}
/// Page through access policies and invoke ArchiveAccessPolicy for each record.
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
            let command = authorization::ArchiveAccessPolicyRequest {
                access_policy_id: item.access_policy_id,
                context: Some(context.clone()),
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.archive_access_policy(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through csrs and invoke ActivateCsr for each record.
async fn sweep_csr(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = CsrWorkflowServiceClient::new(channel.clone());
    let mut writer = CsrLifecycleServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = certificates::ListCsrWorkflowsRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.list_csr_workflows(page).await?.into_inner();
        for item in page.csrs {
            let command = certificates::ActivateCsrRequest {
                context: Some(context.clone()),
                csr_id: item.csr_id,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.activate_csr(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through config changes and invoke ActivateConfigChange for each record.
async fn sweep_config_change(
    channel: tonic::transport::Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut reader = ConfigChangeQueryServiceClient::new(channel.clone());
    let mut writer = ConfigChangeLifecycleServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = configuration::BrowseConfigChangesRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.browse_config_changes(page).await?.into_inner();
        for item in page.config_changes {
            let command = configuration::ActivateConfigChangeRequest {
                config_change_id: item.config_change_id,
                context: Some(context.clone()),
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.activate_config_change(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through feature flags and invoke ArchiveFeatureFlag for each record.
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
            let command = featureflags::ArchiveFeatureFlagRequest {
                context: Some(context.clone()),
                feature_flag_id: item.feature_flag_id,
                ..Default::default()
            };
            let request = acme_platform::auth::service_request(tonic::Request::new(command), &context)?;
            writer.archive_feature_flag(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
