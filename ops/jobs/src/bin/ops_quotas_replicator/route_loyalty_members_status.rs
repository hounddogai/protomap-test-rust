//! `route_loyalty_members_status` reads `acme.loyalty.v1.LoyaltyMemberQueryService/ResolveLoyaltyMembers` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::loyalty::v1 as loyalty;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        loyalty::loyalty_member_query_service_client::LoyaltyMemberQueryServiceClient::new(channel.clone());
    let fetched = source
        .resolve_loyalty_members(loyalty::ResolveLoyaltyMembersRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = LoyaltyMembersStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
    };
    deliver_loyalty_members_status("config-set-workflow", record, channel, &context).await
}

/// What `route_loyalty_members_status` keeps of each ResolveLoyaltyMembers response.
#[derive(Clone, Debug, Default)]
pub struct LoyaltyMembersStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `route_loyalty_members_status` hands each record.
trait LoyaltyMembersStatusTarget {
    fn apply(&self, record: LoyaltyMembersStatusRecord) -> Delivery;
}

/// Passes records to `acme.configuration.v1.ConfigSetWorkflowService`.
struct ConfigSetWorkflowSink {
    client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient<Channel>,
}

impl LoyaltyMembersStatusTarget for ConfigSetWorkflowSink {
    fn apply(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_config_set_workflow(configuration::AdvanceConfigSetWorkflowRequest {
                    config_set_id: record.next_page_token,
                    config_set: Some(configuration::ConfigSet {
                        name: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.PermissionCheckValidationService`.
struct PermissionCheckValidationSink {
    client: authorization::permission_check_validation_service_client::PermissionCheckValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LoyaltyMembersStatusTarget for PermissionCheckValidationSink {
    fn apply(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ExplainPermissionCheckDecisionRequest {
                    reason: record.next_page_token,
                    permission_check: Some(authorization::PermissionCheck {
                        permission_check_id: record.correlation_id,
                        source: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_permission_check_decision(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyApprovalService`.
struct CryptoKeyApprovalSink {
    client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient<Channel>,
}

impl LoyaltyMembersStatusTarget for CryptoKeyApprovalSink {
    fn apply(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delegate_crypto_key_approval(keymanagement::DelegateCryptoKeyApprovalRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        crypto_key_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct LoyaltyMembersStatusLogOnly;

impl LoyaltyMembersStatusTarget for LoyaltyMembersStatusLogOnly {
    fn apply(&self, record: LoyaltyMembersStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_loyalty_members_status(
    kind: &str,
    record: LoyaltyMembersStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn LoyaltyMembersStatusTarget> = match kind {
        "config-set-workflow" => Box::new(ConfigSetWorkflowSink {
            client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        "permission-check-validation" => Box::new(PermissionCheckValidationSink {
            client:
                authorization::permission_check_validation_service_client::PermissionCheckValidationServiceClient::new(
                    channel.clone(),
                ),
            context: context.clone(),
        }),
        "crypto-key-approval" => Box::new(CryptoKeyApprovalSink {
            client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(channel),
        }),
        _ => Box::new(LoyaltyMembersStatusLogOnly),
    };
    sink.apply(record).await
}
