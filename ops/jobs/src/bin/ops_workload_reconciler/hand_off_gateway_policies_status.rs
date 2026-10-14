//! `hand_off_gateway_policies_status` reads
//! `acme.edgegateway.v1.GatewayPolicyQueryService/CountGatewayPolicies` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_query_service_client::GatewayPolicyQueryServiceClient::new(channel.clone());
    let loaded = source
        .count_gateway_policies(edgegateway::CountGatewayPoliciesRequest {
            context: Some(context.clone()),
            order_by: "default-hand-off-gateway-policies-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = OffGatewayPoliciesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn OffGatewayPoliciesStatusDestination>> = vec![
        Box::new(FlagRolloutAdminSink {
            client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(PromotionCalculationSink {
            client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient::new(channel),
        }),
    ];
    sinks[0].deliver(record).await
}

/// What `hand_off_gateway_policies_status` keeps of each CountGatewayPolicies response.
#[derive(Clone, Debug, Default)]
pub struct OffGatewayPoliciesStatusRecord {
    pub status: String,
    pub correlation_id: String,
}

/// Where `hand_off_gateway_policies_status` hands each record.
trait OffGatewayPoliciesStatusDestination {
    fn deliver(&self, record: OffGatewayPoliciesStatusRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutAdminService`.
struct FlagRolloutAdminSink {
    client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffGatewayPoliciesStatusDestination for FlagRolloutAdminSink {
    fn deliver(&self, record: OffGatewayPoliciesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::ArchiveFlagRolloutRequest {
                    flag_rollout_id: record.correlation_id,
                    details: Some(featureflags::archive_flag_rollout_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.archive_flag_rollout(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.promotions.v1.PromotionCalculationService`.
struct PromotionCalculationSink {
    client: promotions::promotion_calculation_service_client::PromotionCalculationServiceClient<Channel>,
}

impl OffGatewayPoliciesStatusDestination for PromotionCalculationSink {
    fn deliver(&self, record: OffGatewayPoliciesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_promotion(promotions::EstimatePromotionRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
