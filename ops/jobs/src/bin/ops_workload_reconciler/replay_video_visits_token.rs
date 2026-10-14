//! `replay_video_visits_token` reads `acme.telehealth.v1.VideoVisitQueryService/BrowseVideoVisits` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::assets::v1 as assets;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient::new(channel.clone());
    let found = source
        .browse_video_visits(telehealth::BrowseVideoVisitsRequest {
            context: Some(context.clone()),
            order_by: "nightly-replay-video-visits-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = found.page_info.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = VideoVisitsTokenRecord {
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
        message: result.message,
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn VideoVisitsTokenTarget>> = BTreeMap::new();
    sinks.insert(
        "fixed-asset-calculation",
        Box::new(FixedAssetCalculationSink {
            client: assets::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "environment-workflow",
        Box::new(EnvironmentWorkflowSink {
            client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "vehicle",
        Box::new(VehicleSink {
            client: fleet::vehicle_service_client::VehicleServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    match sinks.get("fixed-asset-calculation") {
        Some(sink) => sink.send(record).await,
        None => anyhow::bail!("no sink registered for fixed-asset-calculation"),
    }
}

/// What `replay_video_visits_token` keeps of each BrowseVideoVisits response.
#[derive(Clone, Debug, Default)]
pub struct VideoVisitsTokenRecord {
    pub next_page_token: String,
    pub correlation_id: String,
    pub message: String,
    pub status: String,
}

/// Where `replay_video_visits_token` hands each record.
trait VideoVisitsTokenTarget {
    fn send(&self, record: VideoVisitsTokenRecord) -> Delivery;
}

/// Writes records to `acme.assets.v1.FixedAssetCalculationService`.
struct FixedAssetCalculationSink {
    client: assets::fixed_asset_calculation_service_client::FixedAssetCalculationServiceClient<Channel>,
}

impl VideoVisitsTokenTarget for FixedAssetCalculationSink {
    fn send(&self, record: VideoVisitsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_fixed_assets(assets::RecalculateFixedAssetsRequest {
                    fixed_asset: Some(assets::FixedAsset {
                        fixed_asset_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.EnvironmentWorkflowService`.
struct EnvironmentWorkflowSink {
    client: configuration::environment_workflow_service_client::EnvironmentWorkflowServiceClient<Channel>,
}

impl VideoVisitsTokenTarget for EnvironmentWorkflowSink {
    fn send(&self, record: VideoVisitsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .retry_environment_workflow(configuration::RetryEnvironmentWorkflowRequest {
                    environment_id: record.correlation_id,
                    environment: Some(configuration::Environment {
                        code: record.message,
                        ..Default::default()
                    }),
                    details: Some(configuration::retry_environment_workflow_request::Details {
                        initiated_by: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
    context: common::RequestContext,
}

impl VideoVisitsTokenTarget for VehicleSink {
    fn send(&self, record: VideoVisitsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::CloneVehicleRequest {
                    vehicle: Some(fleet::Vehicle {
                        code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_vehicle(request).await?;
            Ok(())
        })
    }
}
