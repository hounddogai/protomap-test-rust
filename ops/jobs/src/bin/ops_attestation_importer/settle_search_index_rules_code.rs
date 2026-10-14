//! `settle_search_index_rules_code` reads `acme.search.v1.SearchIndexValidationService/GetSearchIndexRules`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        search::search_index_validation_service_client::SearchIndexValidationServiceClient::new(channel.clone());
    let found = source
        .get_search_index_rules(search::GetSearchIndexRulesRequest {
            context: Some(context.clone()),
            search_index_id: "main-settle-search-index-rules-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let search_index = found.search_index.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = SearchIndexRulesCodeRecord {
        code: search_index.code,
        correlation_id: result.correlation_id,
        message: result.message,
        name: search_index.name,
    };
    let sinks: Vec<(&str, Box<dyn SearchIndexRulesCodeTarget>)> = vec![
        (
            "config-change-lifecycle",
            Box::new(ConfigChangeLifecycleSink {
                client: configuration::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient::new(
                    channel.clone(),
                ),
                context: context.clone(),
            }),
        ),
        (
            "notification-validation",
            Box::new(NotificationValidationSink {
                client: notifications::notification_validation_service_client::NotificationValidationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "config-set-calculation",
            Box::new(ConfigSetCalculationSink {
                client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "config-change-lifecycle" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `settle_search_index_rules_code` keeps of each GetSearchIndexRules response.
#[derive(Clone, Debug, Default)]
pub struct SearchIndexRulesCodeRecord {
    pub code: String,
    pub correlation_id: String,
    pub message: String,
    pub name: String,
}

/// Where `settle_search_index_rules_code` hands each record.
trait SearchIndexRulesCodeTarget {
    fn push(&self, record: SearchIndexRulesCodeRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.ConfigChangeLifecycleService`.
struct ConfigChangeLifecycleSink {
    client: configuration::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl SearchIndexRulesCodeTarget for ConfigChangeLifecycleSink {
    fn push(&self, record: SearchIndexRulesCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ReinstateConfigChangeRequest {
                    details: Some(configuration::reinstate_config_change_request::Details {
                        reason_code: record.correlation_id,
                        ..Default::default()
                    }),
                    config_change_id: record.name,
                    reason: record.code,
                    ..Default::default()
                }),
                &context,
            )?;
            client.reinstate_config_change(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.notifications.v1.NotificationValidationService`.
struct NotificationValidationSink {
    client: notifications::notification_validation_service_client::NotificationValidationServiceClient<Channel>,
}

impl SearchIndexRulesCodeTarget for NotificationValidationSink {
    fn push(&self, record: SearchIndexRulesCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_notification_decision(notifications::ExplainNotificationDecisionRequest {
                    notification_id: record.code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SearchIndexRulesCodeTarget for ConfigSetCalculationSink {
    fn push(&self, record: SearchIndexRulesCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::CalculateConfigSetRequest {
                    config_set: Some(configuration::ConfigSet {
                        name: record.message,
                        config_set_id: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(configuration::calculate_config_set_request::Details {
                        currency_code: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_config_set(request).await?;
            Ok(())
        })
    }
}
