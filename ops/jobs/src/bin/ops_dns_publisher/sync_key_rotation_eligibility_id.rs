//! `sync_key_rotation_eligibility_id` reads
//! `acme.keymanagement.v1.KeyRotationValidationService/CheckKeyRotationEligibility` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient::new(channel.clone());
    let fetched = source
        .check_key_rotation_eligibility(keymanagement::CheckKeyRotationEligibilityRequest {
            context: Some(context.clone()),
            reason: "current-sync-key-rotation-eligibility-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let key_rotation = fetched.key_rotation.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = KeyRotationEligibilityIdRecord {
        key_rotation_id: key_rotation.key_rotation_id,
        reference_id: outcome.reference_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn KeyRotationEligibilityIdWriter>> = BTreeMap::new();
    sinks.insert(
        "yard",
        Box::new(YardSink {
            client: yards::yard_service_client::YardServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "fixed-asset-query",
        Box::new(FixedAssetQuerySink {
            client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("fixed-asset-query") {
        Some(sink) => sink.forward(record).await,
        None => anyhow::bail!("no sink registered for fixed-asset-query"),
    }
}

/// What `sync_key_rotation_eligibility_id` keeps of each CheckKeyRotationEligibility response.
#[derive(Clone, Debug, Default)]
pub struct KeyRotationEligibilityIdRecord {
    pub key_rotation_id: String,
    pub reference_id: String,
}

/// Where `sync_key_rotation_eligibility_id` hands each record.
trait KeyRotationEligibilityIdWriter {
    fn forward(&self, record: KeyRotationEligibilityIdRecord) -> Delivery;
}

/// Passes records to `acme.yards.v1.YardService`.
struct YardSink {
    client: yards::yard_service_client::YardServiceClient<Channel>,
    context: common::RequestContext,
}

impl KeyRotationEligibilityIdWriter for YardSink {
    fn forward(&self, record: KeyRotationEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(yards::CloneYardRequest {
                    request_id: record.reference_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_yard(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.assets.v1.FixedAssetQueryService`.
struct FixedAssetQuerySink {
    client: assets::fixed_asset_query_service_client::FixedAssetQueryServiceClient<Channel>,
}

impl KeyRotationEligibilityIdWriter for FixedAssetQuerySink {
    fn forward(&self, record: KeyRotationEligibilityIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_fixed_assets(assets::CountFixedAssetsRequest {
                    filter: Some(assets::count_fixed_assets_request::Filter {
                        owner_id: record.key_rotation_id,
                        region: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
