//! `push_seat_map_message` reads `acme.seating.v1.SeatMapCalculationService/CalculateSeatMap` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::seating::v1 as seating;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel.clone());
    let loaded = source
        .calculate_seat_map(seating::CalculateSeatMapRequest {
            context: Some(context.clone()),
            seat_map_id: "live-push-seat-map-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = SeatMapMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn SeatMapMessageWriter>> = vec![
        Box::new(FlagEvaluationAdminSink {
            client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(FlagRolloutCalculationSink {
            client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
                channel,
            ),
        }),
    ];
    sinks[0].push(record).await
}

/// What `push_seat_map_message` keeps of each CalculateSeatMap response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `push_seat_map_message` hands each record.
trait SeatMapMessageWriter {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery;
}

/// Sends records to `acme.featureflags.v1.FlagEvaluationAdminService`.
struct FlagEvaluationAdminSink {
    client: featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient<Channel>,
}

impl SeatMapMessageWriter for FlagEvaluationAdminSink {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .purge_flag_evaluation(featureflags::PurgeFlagEvaluationRequest {
                    etag: record.correlation_id,
                    flag_evaluation_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
}

impl SeatMapMessageWriter for AccessGrantValidationSink {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_access_grants(secrets::LintAccessGrantsRequest {
                    access_grant: Some(secrets::AccessGrant {
                        access_grant_id: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
}

impl SeatMapMessageWriter for FlagRolloutCalculationSink {
    fn push(&self, record: SeatMapMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_flag_rollout_calculation(featureflags::ExplainFlagRolloutCalculationRequest {
                    reason: record.message,
                    flag_rollout_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
