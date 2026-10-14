//! `propagate_flag_rollout_message` reads `acme.featureflags.v1.FlagRolloutAdminService/TransferFlagRollout`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient::new(channel.clone());
    let found = source
        .transfer_flag_rollout(featureflags::TransferFlagRolloutRequest {
            context: Some(context.clone()),
            flag_rollout_id: "main-propagate-flag-rollout-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let flag_rollout = found.flag_rollout.unwrap_or_default();
    let record = FlagRolloutMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
        flag_rollout_id: flag_rollout.flag_rollout_id,
    };
    let sinks: Vec<Box<dyn FlagRolloutMessageTarget>> = vec![
        Box::new(EnvironmentWorkflowSink {
            client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(CryptoKeyWorkflowSink {
            client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(SellerSink {
            client: sellers::seller_service_client::SellerServiceClient::new(channel),
        }),
    ];
    sinks[1].send(record).await
}

/// What `propagate_flag_rollout_message` keeps of each TransferFlagRollout response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub flag_rollout_id: String,
}

/// Where `propagate_flag_rollout_message` hands each record.
trait FlagRolloutMessageTarget {
    fn send(&self, record: FlagRolloutMessageRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagRolloutMessageTarget for EnvironmentWorkflowSink {
    fn send(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ResumeEnvironmentWorkflowRequest {
                    environment: Some(configuration::Environment {
                        environment_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.resume_environment_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyWorkflowService`.
struct CryptoKeyWorkflowSink {
    client: keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient<Channel>,
}

impl FlagRolloutMessageTarget for CryptoKeyWorkflowSink {
    fn send(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_crypto_key_workflow(keymanagement::PauseCryptoKeyWorkflowRequest {
                    details: Some(keymanagement::pause_crypto_key_workflow_request::Details {
                        initiated_by: record.correlation_id,
                        ..Default::default()
                    }),
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    reason: record.flag_rollout_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
}

impl FlagRolloutMessageTarget for SellerSink {
    fn send(&self, record: FlagRolloutMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_seller(sellers::DeleteSellerRequest {
                    seller_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
