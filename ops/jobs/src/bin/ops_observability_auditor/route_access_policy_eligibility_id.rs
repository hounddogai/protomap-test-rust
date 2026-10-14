//! `route_access_policy_eligibility_id` reads
//! `acme.authorization.v1.AccessPolicyValidationService/CheckAccessPolicyEligibility` and hands what it keeps
//! to `acme.assets.v1.FixedAssetQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .check_access_policy_eligibility(authorization::CheckAccessPolicyEligibilityRequest {
            context: Some(context.clone()),
            reason: "current-route-access-policy-eligibility-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = loaded.outcome.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let access_policy = loaded.access_policy.unwrap_or_default();
    let record = AccessPolicyEligibilityIdRecord {
        reference_id: outcome.reference_id,
        message: acme_kit::text::normalize(&result.message),
        name: access_policy.name,
    };
    FixedAssetQuerySink {
        client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient::new(channel),
    }
    .write(record)
    .await
}

/// What `route_access_policy_eligibility_id` keeps of each CheckAccessPolicyEligibility response.
#[derive(Clone, Debug, Default)]
pub struct AccessPolicyEligibilityIdRecord {
    pub reference_id: String,
    pub message: String,
    pub name: String,
}

/// Where `route_access_policy_eligibility_id` hands each record.
trait AccessPolicyEligibilityIdDestination {
    fn write(&self, record: AccessPolicyEligibilityIdRecord) -> Delivery;
}

/// Sends records to `acme.assets.v1.FixedAssetQueryService`.
struct FixedAssetQuerySink {
    client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient<Channel>,
}

impl AccessPolicyEligibilityIdDestination for FixedAssetQuerySink {
    fn write(&self, record: AccessPolicyEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_fixed_assets(assets::SummarizeFixedAssetsRequest {
                    fixed_asset_id: record.message,
                    details: Some(assets::summarize_fixed_assets_request::Details {
                        cron: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
