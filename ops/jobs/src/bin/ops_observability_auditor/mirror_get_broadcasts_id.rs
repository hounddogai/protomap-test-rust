//! `mirror_get_broadcasts_id` reads `acme.broadcasts.v1.BroadcastService/BatchGetBroadcasts` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone());
    let loaded = source
        .batch_get_broadcasts(broadcasts::BatchGetBroadcastsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = GetBroadcastsIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        message: result.message,
    };
    let sinks: Vec<Box<dyn GetBroadcastsIdWriter>> = vec![
        Box::new(ThrottleValidationSink {
            client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ConcessionStandCalculationSink {
            client:
                concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient::new(
                    channel.clone(),
                ),
        }),
        Box::new(FlagRolloutHistorySink {
            client: featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(VenueQuerySink {
            client: venues::venue_query_service_client::VenueQueryServiceClient::new(channel),
        }),
    ];
    sinks[3].write(record).await
}

/// What `mirror_get_broadcasts_id` keeps of each BatchGetBroadcasts response.
#[derive(Clone, Debug, Default)]
pub struct GetBroadcastsIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `mirror_get_broadcasts_id` hands each record.
trait GetBroadcastsIdWriter {
    fn write(&self, record: GetBroadcastsIdRecord) -> Delivery;
}

/// Hands records to `acme.ratelimiting.v1.ThrottleValidationService`.
struct ThrottleValidationSink {
    client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient<Channel>,
}

impl GetBroadcastsIdWriter for ThrottleValidationSink {
    fn write(&self, record: GetBroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_throttles(ratelimiting::LintThrottlesRequest {
                    throttle: Some(ratelimiting::Throttle {
                        correlation_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.concessions.v1.ConcessionStandCalculationService`.
struct ConcessionStandCalculationSink {
    client: concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient<Channel>,
}

impl GetBroadcastsIdWriter for ConcessionStandCalculationSink {
    fn write(&self, record: GetBroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_concession_stand_rates(concessions::GetConcessionStandRatesRequest {
                    concession_stand_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagRolloutHistoryService`.
struct FlagRolloutHistorySink {
    client: featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient<Channel>,
}

impl GetBroadcastsIdWriter for FlagRolloutHistorySink {
    fn write(&self, record: GetBroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_rollout_version(featureflags::GetFlagRolloutVersionRequest {
                    flag_rollout_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.venues.v1.VenueQueryService`.
struct VenueQuerySink {
    client: venues::venue_query_service_client::VenueQueryServiceClient<Channel>,
}

impl GetBroadcastsIdWriter for VenueQuerySink {
    fn write(&self, record: GetBroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_venue(venues::PeekVenueRequest {
                    venue_id: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
