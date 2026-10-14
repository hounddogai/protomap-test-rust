//! `export_dispatch_order_notes` reads `acme.dispatch.v1.DispatchOrderService/UpdateDispatchOrder` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(channel.clone());
    let reply = source
        .update_dispatch_order(dispatch::UpdateDispatchOrderRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let dispatch_order = reply.dispatch_order.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = DispatchOrderNotesRecord {
        notes: dispatch_order.notes,
        message: result.message,
        dispatch_order_id: acme_kit::text::normalize(&dispatch_order.dispatch_order_id),
        status: result.status,
    };
    deliver_dispatch_order_notes("promotion-calculation", record, channel, &context).await
}

/// What `export_dispatch_order_notes` keeps of each UpdateDispatchOrder response.
#[derive(Clone, Debug, Default)]
pub struct DispatchOrderNotesRecord {
    pub notes: String,
    pub message: String,
    pub dispatch_order_id: String,
    pub status: String,
}

/// Where `export_dispatch_order_notes` hands each record.
trait DispatchOrderNotesWriter {
    fn write(&self, record: DispatchOrderNotesRecord) -> Delivery;
}

/// Writes records to `acme.promotions.v1.PromotionCalculationService`.
struct PromotionCalculationSink {
    client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient<Channel>,
}

impl DispatchOrderNotesWriter for PromotionCalculationSink {
    fn write(&self, record: DispatchOrderNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_promotion(promotions::CalculatePromotionRequest {
                    reason: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigSetWorkflowService`.
struct ConfigSetWorkflowSink {
    client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient<Channel>,
}

impl DispatchOrderNotesWriter for ConfigSetWorkflowSink {
    fn write(&self, record: DispatchOrderNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_config_set_workflow(configuration::PauseConfigSetWorkflowRequest {
                    config_set: Some(configuration::ConfigSet {
                        name: record.message,
                        ..Default::default()
                    }),
                    details: Some(configuration::pause_config_set_workflow_request::Details {
                        initiated_by: record.status,
                        ..Default::default()
                    }),
                    config_set_id: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketWorkflowService`.
struct QuotaBucketWorkflowSink {
    client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient<Channel>,
}

impl DispatchOrderNotesWriter for QuotaBucketWorkflowSink {
    fn write(&self, record: DispatchOrderNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resume_quota_bucket_workflow(ratelimiting::ResumeQuotaBucketWorkflowRequest {
                    details: Some(ratelimiting::resume_quota_bucket_workflow_request::Details {
                        initiated_by: record.notes,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct DispatchOrderNotesLogOnly;

impl DispatchOrderNotesWriter for DispatchOrderNotesLogOnly {
    fn write(&self, record: DispatchOrderNotesRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.notes);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_dispatch_order_notes(
    kind: &str,
    record: DispatchOrderNotesRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn DispatchOrderNotesWriter> = match kind {
        "promotion-calculation" => Box::new(PromotionCalculationSink {
            client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "config-set-workflow" => Box::new(ConfigSetWorkflowSink {
            client: configuration::config_set_workflow_service_client::ConfigSetWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
        "quota-bucket-workflow" => Box::new(QuotaBucketWorkflowSink {
            client: ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(channel),
        }),
        _ => Box::new(DispatchOrderNotesLogOnly),
    };
    sink.write(record).await
}
