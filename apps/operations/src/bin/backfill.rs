//! Scheduled import operations, isolated from the domain server's request handling.

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::authorization::v1::access_policy_service_client::AccessPolicyServiceClient;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::certificates::v1::certificate_service_client::CertificateServiceClient;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::configuration::v1::config_set_service_client::ConfigSetServiceClient;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::edgegateway::v1::api_key_integration_service_client::ApiKeyIntegrationServiceClient;
use acme_proto::acme::edgegateway::v1::api_key_service_client::ApiKeyServiceClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    acme_platform::config::logging();
    let channel = acme_operations::channel()?;
    let context = acme_operations::context()?;
    sweep_access_policy(channel.clone(), context.clone()).await?;
    sweep_certificate(channel.clone(), context.clone()).await?;
    sweep_config_set(channel.clone(), context.clone()).await?;
    sweep_api_key(channel.clone(), context.clone()).await?;
    Ok(())
}
/// Page through access policies and invoke BatchCreateAccessPolicies for each record.
async fn sweep_access_policy(
    channel: tonic::transport::Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut reader = AccessPolicyServiceClient::new(channel.clone());
    let mut writer = AccessPolicyServiceClient::new(channel);
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
            let command = authorization::BatchCreateAccessPoliciesRequest {
                context: Some(context.clone()),
                chunk: Some(authorization::batch_create_access_policies_request::Chunk::Record(
                    (Some(item)).unwrap_or_default(),
                )),
                ..Default::default()
            };
            let frames = tokio_stream::iter([command]);
            let request = acme_platform::auth::service_request(tonic::Request::new(frames), &context)?;
            writer.batch_create_access_policies(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through certificates and invoke BatchCreateCertificates for each record.
async fn sweep_certificate(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = CertificateServiceClient::new(channel.clone());
    let mut writer = CertificateServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = certificates::ListCertificatesRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.list_certificates(page).await?.into_inner();
        for item in page.certificates {
            let command = certificates::BatchCreateCertificatesRequest {
                context: Some(context.clone()),
                chunk: Some(certificates::batch_create_certificates_request::Chunk::Record(
                    (Some(item)).unwrap_or_default(),
                )),
                ..Default::default()
            };
            let frames = tokio_stream::iter([command]);
            let request = acme_platform::auth::service_request(tonic::Request::new(frames), &context)?;
            writer.batch_create_certificates(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through config sets and invoke BatchCreateConfigSets for each record.
async fn sweep_config_set(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = ConfigSetServiceClient::new(channel.clone());
    let mut writer = ConfigSetServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = configuration::SearchConfigSetsRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.search_config_sets(page).await?.into_inner();
        for item in page.config_sets {
            let command = configuration::BatchCreateConfigSetsRequest {
                context: Some(context.clone()),
                chunk: Some(configuration::batch_create_config_sets_request::Chunk::Record(
                    (Some(item)).unwrap_or_default(),
                )),
                ..Default::default()
            };
            let frames = tokio_stream::iter([command]);
            let request = acme_platform::auth::service_request(tonic::Request::new(frames), &context)?;
            writer.batch_create_config_sets(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
/// Page through api keys and invoke IngestApiKeyEvents for each record.
async fn sweep_api_key(channel: tonic::transport::Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut reader = ApiKeyServiceClient::new(channel.clone());
    let mut writer = ApiKeyIntegrationServiceClient::new(channel);
    let mut token = String::new();
    loop {
        let page = edgegateway::ListApiKeysRequest {
            context: Some(context.clone()),
            page: Some(common::PageRequest {
                page_size: 100,
                page_token: token,
            }),
            ..Default::default()
        };
        let page = acme_platform::auth::service_request(tonic::Request::new(page), &context)?;
        let page = reader.list_api_keys(page).await?.into_inner();
        for item in page.api_keys {
            let command = edgegateway::IngestApiKeyEventsRequest {
                context: Some(context.clone()),
                chunk: Some(edgegateway::ingest_api_key_events_request::Chunk::Record(
                    (Some(item)).unwrap_or_default(),
                )),
                ..Default::default()
            };
            let frames = tokio_stream::iter([command]);
            let request = acme_platform::auth::service_request(tonic::Request::new(frames), &context)?;
            writer.ingest_api_key_events(request).await?;
        }
        token = page.page_info.unwrap_or_default().next_page_token;
        if token.is_empty() {
            break;
        }
    }
    Ok(())
}
