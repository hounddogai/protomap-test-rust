//! `push_subscriptions_token` reads `acme.subscriptions.v1.SubscriptionService/ListSubscriptions` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::subscriptions::v1 as subscriptions;
use acme_proto::acme::treasury::v1 as treasury;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = subscriptions::subscription_service_client::SubscriptionServiceClient::new(channel.clone());
    let reply = source
        .list_subscriptions(subscriptions::ListSubscriptionsRequest {
            context: Some(context.clone()),
            order_by: "live-push-subscriptions-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = reply.page_info.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = SubscriptionsTokenRecord {
        next_page_token: page_info.next_page_token,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn SubscriptionsTokenTarget>> = BTreeMap::new();
    sinks.insert(
        "cash-position-calculation",
        Box::new(CashPositionCalculationSink {
            client: treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "config-change-approval",
        Box::new(ConfigChangeApprovalSink {
            client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "role-binding-validation",
        Box::new(RoleBindingValidationSink {
            client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    );
    let Some(sink) = sinks.get("config-change-approval") else {
        anyhow::bail!("no sink registered for config-change-approval");
    };
    sink.push(record).await
}

/// What `push_subscriptions_token` keeps of each ListSubscriptions response.
#[derive(Clone, Debug, Default)]
pub struct SubscriptionsTokenRecord {
    pub next_page_token: String,
    pub message: String,
}

/// Where `push_subscriptions_token` hands each record.
trait SubscriptionsTokenTarget {
    fn push(&self, record: SubscriptionsTokenRecord) -> Delivery;
}

/// Writes records to `acme.treasury.v1.CashPositionCalculationService`.
struct CashPositionCalculationSink {
    client: treasury::cash_position_calculation_service_client::CashPositionCalculationServiceClient<Channel>,
}

impl SubscriptionsTokenTarget for CashPositionCalculationSink {
    fn push(&self, record: SubscriptionsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_cash_position_options(treasury::CompareCashPositionOptionsRequest {
                    cash_position: Some(treasury::CashPosition {
                        currency_code: record.next_page_token,
                        ..Default::default()
                    }),
                    cash_position_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.configuration.v1.ConfigChangeApprovalService`.
struct ConfigChangeApprovalSink {
    client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl SubscriptionsTokenTarget for ConfigChangeApprovalSink {
    fn push(&self, record: SubscriptionsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::RejectConfigChangeRequest {
                    details: Some(configuration::reject_config_change_request::Details {
                        comment: record.next_page_token,
                        ..Default::default()
                    }),
                    config_change_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.reject_config_change(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.RoleBindingValidationService`.
struct RoleBindingValidationSink {
    client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SubscriptionsTokenTarget for RoleBindingValidationSink {
    fn push(&self, record: SubscriptionsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::NormalizeRoleBindingRequest {
                    role_binding_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.normalize_role_binding(request).await?;
            Ok(())
        })
    }
}
