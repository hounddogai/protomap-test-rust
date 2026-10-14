//! `forward_throttle_approval_status` reads
//! `acme.ratelimiting.v1.ThrottleApprovalService/DelegateThrottleApproval` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::promotions::v1 as promotions;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_approval_service_client::ThrottleApprovalServiceClient::new(channel.clone());
    let loaded = source
        .delegate_throttle_approval(ratelimiting::DelegateThrottleApprovalRequest {
            context: Some(context.clone()),
            reason: "main-forward-throttle-approval-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let throttle = loaded.throttle.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = ThrottleApprovalStatusRecord {
        status: result.status,
        source: throttle.source,
        reference_id: outcome.reference_id,
    };
    deliver_throttle_approval_status("rate-limit-validation", record, channel, &context).await
}

/// What `forward_throttle_approval_status` keeps of each DelegateThrottleApproval response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleApprovalStatusRecord {
    pub status: String,
    pub source: String,
    pub reference_id: String,
}

/// Where `forward_throttle_approval_status` hands each record.
trait ThrottleApprovalStatusTarget {
    fn forward(&self, record: ThrottleApprovalStatusRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl ThrottleApprovalStatusTarget for RateLimitValidationSink {
    fn forward(&self, record: ThrottleApprovalStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_rate_limit_rules(ratelimiting::GetRateLimitRulesRequest {
                    rate_limit_id: record.reference_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.promotions.v1.PromotionCalculationService`.
struct PromotionCalculationSink {
    client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleApprovalStatusTarget for PromotionCalculationSink {
    fn forward(&self, record: ThrottleApprovalStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(promotions::SimulatePromotionRequest {
                    promotion: Some(promotions::Promotion {
                        name: record.source,
                        promotion_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_promotion(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ThrottleApprovalStatusLogOnly;

impl ThrottleApprovalStatusTarget for ThrottleApprovalStatusLogOnly {
    fn forward(&self, record: ThrottleApprovalStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_throttle_approval_status(
    kind: &str,
    record: ThrottleApprovalStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn ThrottleApprovalStatusTarget> = match kind {
        "rate-limit-validation" => Box::new(RateLimitValidationSink {
            client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "promotion-calculation" => Box::new(PromotionCalculationSink {
            client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(ThrottleApprovalStatusLogOnly),
    };
    sink.forward(record).await
}
