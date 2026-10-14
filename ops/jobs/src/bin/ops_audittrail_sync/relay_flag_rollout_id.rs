//! `relay_flag_rollout_id` reads `acme.featureflags.v1.FlagRolloutCalculationService/EstimateFlagRollout` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .estimate_flag_rollout(featureflags::EstimateFlagRolloutRequest {
            context: Some(context.clone()),
            reason: "main-relay-flag-rollout-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_rollout = loaded.flag_rollout.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let record = FlagRolloutIdRecord {
        flag_rollout_id: flag_rollout.flag_rollout_id,
        message: acme_kit::text::normalize(&result.message),
        currency_code: breakdown.currency_code,
        title: acme_kit::text::normalize(&flag_rollout.title),
    };
    let mut sinks: BTreeMap<String, Box<dyn FlagRolloutIdSink>> = BTreeMap::new();
    sinks.insert(
        "quota-bucket-reporting".to_owned(),
        Box::new(QuotaBucketReportingSink {
            client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "fulfillment-query".to_owned(),
        Box::new(FulfillmentQuerySink {
            client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "work-order-calculation".to_owned(),
        Box::new(WorkOrderCalculationSink {
            client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(channel),
        }),
    );
    let target = std::env::var("RELAY_FLAG_ROLLOUT_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.deliver(record).await,
        None => Ok(()),
    }
}

/// What `relay_flag_rollout_id` keeps of each EstimateFlagRollout response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutIdRecord {
    pub flag_rollout_id: String,
    pub message: String,
    pub currency_code: String,
    pub title: String,
}

/// Where `relay_flag_rollout_id` hands each record.
trait FlagRolloutIdSink {
    fn deliver(&self, record: FlagRolloutIdRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketReportingService`.
struct QuotaBucketReportingSink {
    client: ratelimiting::quota_bucket_reporting_service_client::QuotaBucketReportingServiceClient<Channel>,
}

impl FlagRolloutIdSink for QuotaBucketReportingSink {
    fn deliver(&self, record: FlagRolloutIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .fetch_quota_bucket_report(ratelimiting::FetchQuotaBucketReportRequest {
                    quota_bucket_id: record.title,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fulfillment.v1.FulfillmentQueryService`.
struct FulfillmentQuerySink {
    client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient<Channel>,
}

impl FlagRolloutIdSink for FulfillmentQuerySink {
    fn deliver(&self, record: FlagRolloutIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_fulfillment(fulfillment::PeekFulfillmentRequest {
                    fulfillment_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.maintenance.v1.WorkOrderCalculationService`.
struct WorkOrderCalculationSink {
    client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient<Channel>,
}

impl FlagRolloutIdSink for WorkOrderCalculationSink {
    fn deliver(&self, record: FlagRolloutIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_work_order_calculation(maintenance::ExplainWorkOrderCalculationRequest {
                    work_order: Some(maintenance::WorkOrder {
                        work_order_id: record.message,
                        ..Default::default()
                    }),
                    details: Some(maintenance::explain_work_order_calculation_request::Details {
                        locale: record.flag_rollout_id,
                        ..Default::default()
                    }),
                    reason: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
