//! `backfill_permission_check_eligibility_status` reads
//! `acme.authorization.v1.PermissionCheckValidationService/CheckPermissionCheckEligibility` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::permission_check_validation_service_client::PermissionCheckValidationServiceClient::new(
            channel.clone(),
        );
    let loaded = source
        .check_permission_check_eligibility(authorization::CheckPermissionCheckEligibilityRequest {
            context: Some(context.clone()),
            reason: "primary-backfill-permission-check-eligibility-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = loaded.outcome.unwrap_or_default();
    let permission_check = loaded.permission_check.unwrap_or_default();
    let record = PermissionCheckEligibilityStatusRecord {
        status: outcome.status,
        permission_check_id: permission_check.permission_check_id,
        source: permission_check.source,
    };
    act_on_permission_check_eligibility_status("seat-map", record, channel, context).await
}

/// What `backfill_permission_check_eligibility_status` keeps of each CheckPermissionCheckEligibility response.
#[derive(Clone, Debug, Default)]
pub struct PermissionCheckEligibilityStatusRecord {
    pub status: String,
    pub permission_check_id: String,
    pub source: String,
}

/// Where `backfill_permission_check_eligibility_status` hands each record.
trait PermissionCheckEligibilityStatusDestination {
    fn deliver(&self, record: PermissionCheckEligibilityStatusRecord) -> Delivery;
}

/// Writes records to `acme.seating.v1.SeatMapService`.
struct SeatMapSink {
    client: seating::seat_map_service_client::SeatMapServiceClient<Channel>,
}

impl PermissionCheckEligibilityStatusDestination for SeatMapSink {
    fn deliver(&self, record: PermissionCheckEligibilityStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_seat_map(seating::CreateSeatMapRequest {
                    seat_map: Some(seating::SeatMap {
                        name: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRingValidationService`.
struct KeyRingValidationSink {
    client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl PermissionCheckEligibilityStatusDestination for KeyRingValidationSink {
    fn deliver(&self, record: PermissionCheckEligibilityStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::CheckKeyRingEligibilityRequest {
                    key_ring_id: record.status,
                    details: Some(keymanagement::check_key_ring_eligibility_request::Details {
                        locale: record.source,
                        ..Default::default()
                    }),
                    key_ring: Some(keymanagement::KeyRing {
                        parent_id: record.permission_check_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.check_key_ring_eligibility(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationHistoryService`.
struct FlagEvaluationHistorySink {
    client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient<Channel>,
}

impl PermissionCheckEligibilityStatusDestination for FlagEvaluationHistorySink {
    fn deliver(&self, record: PermissionCheckEligibilityStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .revert_flag_evaluation(featureflags::RevertFlagEvaluationRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_permission_check_eligibility_status(
    kind: &str,
    record: PermissionCheckEligibilityStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(PermissionCheckEligibilityStatusRecord) -> Delivery>> = BTreeMap::new();
    let seat_map_channel = channel.clone();
    actions.insert(
        "seat-map",
        Box::new(move |record| {
            SeatMapSink {
                client: seating::seat_map_service_client::SeatMapServiceClient::new(seat_map_channel.clone()),
            }
            .deliver(record)
        }),
    );
    let key_ring_validation_channel = channel.clone();
    let key_ring_validation_context = context.clone();
    actions.insert(
        "key-ring-validation",
        Box::new(move |record| {
            KeyRingValidationSink {
                client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(
                    key_ring_validation_channel.clone(),
                ),
                context: key_ring_validation_context.clone(),
            }
            .deliver(record)
        }),
    );
    let flag_evaluation_history_channel = channel.clone();
    actions.insert(
        "flag-evaluation-history",
        Box::new(move |record| {
            FlagEvaluationHistorySink {
                client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient::new(
                    flag_evaluation_history_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
