//! `sync_config_change_id` reads `acme.configuration.v1.ConfigChangeCalculationService/QuoteConfigChange` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::payroll::v1 as payroll;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .quote_config_change(configuration::QuoteConfigChangeRequest {
            context: Some(context.clone()),
            config_change_id: "primary-sync-config-change-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_change = loaded.config_change.unwrap_or_default();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let record = ConfigChangeIdRecord {
        correlation_id: config_change.correlation_id,
        kind: config_change.kind,
        config_change_id: acme_kit::text::normalize(&config_change.config_change_id),
        currency_code: breakdown.currency_code,
    };
    dispatch_config_change_id("rate-limit-workflow", record, channel, &context).await
}

/// What `sync_config_change_id` keeps of each QuoteConfigChange response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeIdRecord {
    pub correlation_id: String,
    pub kind: String,
    pub config_change_id: String,
    pub currency_code: String,
}

/// Where `sync_config_change_id` hands each record.
trait ConfigChangeIdSink {
    fn forward(&self, record: ConfigChangeIdRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangeIdSink for RateLimitWorkflowSink {
    fn forward(&self, record: ConfigChangeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::StartRateLimitWorkflowRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::start_rate_limit_workflow_request::Details {
                        step: record.config_change_id,
                        ..Default::default()
                    }),
                    reason: record.kind,
                    ..Default::default()
                }),
                &context,
            )?;
            client.start_rate_limit_workflow(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.edgegateway.v1.ApiKeyWorkflowService`.
struct ApiKeyWorkflowSink {
    client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient<Channel>,
}

impl ConfigChangeIdSink for ApiKeyWorkflowSink {
    fn forward(&self, record: ConfigChangeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .start_api_key_workflow(edgegateway::StartApiKeyWorkflowRequest {
                    details: Some(edgegateway::start_api_key_workflow_request::Details {
                        initiated_by: record.correlation_id,
                        step: record.currency_code,
                        ..Default::default()
                    }),
                    api_key_id: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.payroll.v1.PayRunService`.
struct PayRunSink {
    client: payroll::pay_run_service_client::PayRunServiceClient<Channel>,
}

impl ConfigChangeIdSink for PayRunSink {
    fn forward(&self, record: ConfigChangeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_pay_run(payroll::UpdatePayRunRequest {
                    pay_run: Some(payroll::PayRun {
                        pay_run_id: record.config_change_id,
                        number: record.kind,
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
struct ConfigChangeIdLogOnly;

impl ConfigChangeIdSink for ConfigChangeIdLogOnly {
    fn forward(&self, record: ConfigChangeIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_config_change_id(
    kind: &str,
    record: ConfigChangeIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let rate_limit_workflow = RateLimitWorkflowSink {
        client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let api_key_workflow = ApiKeyWorkflowSink {
        client: edgegateway::api_key_workflow_service_client::ApiKeyWorkflowServiceClient::new(channel.clone()),
    };
    let pay_run = PayRunSink {
        client: payroll::pay_run_service_client::PayRunServiceClient::new(channel),
    };
    let write: &dyn Fn(ConfigChangeIdRecord) -> Delivery = match kind {
        "rate-limit-workflow" => &|record| rate_limit_workflow.forward(record),
        "api-key-workflow" => &|record| api_key_workflow.forward(record),
        "pay-run" => &|record| pay_run.forward(record),
        _ => &|record| ConfigChangeIdLogOnly.forward(record),
    };
    write(record).await
}
