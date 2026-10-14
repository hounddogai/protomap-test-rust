//! `replay_broadcasts_message` reads `acme.broadcasts.v1.BroadcastQueryService/ResolveBroadcasts` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel.clone());
    let fetched = source
        .resolve_broadcasts(broadcasts::ResolveBroadcastsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = BroadcastsMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
    };
    deliver_broadcasts_message("seat-map-calculation", record, channel, &context).await
}

/// What `replay_broadcasts_message` keeps of each ResolveBroadcasts response.
#[derive(Clone, Debug, Default)]
pub struct BroadcastsMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `replay_broadcasts_message` hands each record.
trait BroadcastsMessageSink {
    fn write(&self, record: BroadcastsMessageRecord) -> Delivery;
}

/// Passes records to `acme.seating.v1.SeatMapCalculationService`.
struct SeatMapCalculationSink {
    client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient<Channel>,
}

impl BroadcastsMessageSink for SeatMapCalculationSink {
    fn write(&self, record: BroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_seat_maps(seating::RecalculateSeatMapsRequest {
                    seat_map_id: record.message,
                    seat_map: Some(seating::SeatMap {
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyApprovalService`.
struct CryptoKeyApprovalSink {
    client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl BroadcastsMessageSink for CryptoKeyApprovalSink {
    fn write(&self, record: BroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::RejectCryptoKeyRequest {
                    details: Some(keymanagement::reject_crypto_key_request::Details {
                        comment: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.reject_crypto_key(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
}

impl BroadcastsMessageSink for FlagEvaluationNotificationSink {
    fn write(&self, record: BroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .send_flag_evaluation_test_notice(featureflags::SendFlagEvaluationTestNoticeRequest {
                    details: Some(featureflags::send_flag_evaluation_test_notice_request::Details {
                        channel: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRingLifecycleService`.
struct KeyRingLifecycleSink {
    client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient<Channel>,
}

impl BroadcastsMessageSink for KeyRingLifecycleSink {
    fn write(&self, record: BroadcastsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reinstate_key_ring(keymanagement::ReinstateKeyRingRequest {
                    reason: record.correlation_id,
                    key_ring_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct BroadcastsMessageLogOnly;

impl BroadcastsMessageSink for BroadcastsMessageLogOnly {
    fn write(&self, record: BroadcastsMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_broadcasts_message(
    kind: &str,
    record: BroadcastsMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn BroadcastsMessageSink> = match kind {
        "seat-map-calculation" => Box::new(SeatMapCalculationSink {
            client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel.clone()),
        }),
        "crypto-key-approval" => Box::new(CryptoKeyApprovalSink {
            client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "flag-evaluation-notification" => Box::new(FlagEvaluationNotificationSink {
            client:
                featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(
                    channel.clone(),
                ),
        }),
        "key-ring-lifecycle" => Box::new(KeyRingLifecycleSink {
            client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient::new(channel),
        }),
        _ => Box::new(BroadcastsMessageLogOnly),
    };
    sink.write(record).await
}
