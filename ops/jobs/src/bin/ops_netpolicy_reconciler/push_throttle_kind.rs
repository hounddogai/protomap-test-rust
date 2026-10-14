//! `push_throttle_kind` reads `acme.ratelimiting.v1.ThrottleValidationService/ScoreThrottle` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(channel.clone());
    let loaded = source
        .score_throttle(ratelimiting::ScoreThrottleRequest {
            context: Some(context.clone()),
            reason: "main-push-throttle-kind".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let throttle = loaded.throttle.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let record = ThrottleKindRecord {
        kind: throttle.kind,
        correlation_id: result.correlation_id,
        message: result.message,
        currency_code: breakdown.currency_code,
    };
    dispatch_throttle_kind("tour", record, channel, &context).await
}

/// What `push_throttle_kind` keeps of each ScoreThrottle response.
#[derive(Clone, Debug, Default)]
pub struct ThrottleKindRecord {
    pub kind: String,
    pub correlation_id: String,
    pub message: String,
    pub currency_code: String,
}

/// Where `push_throttle_kind` hands each record.
trait ThrottleKindSink {
    fn push(&self, record: ThrottleKindRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRingValidationService`.
struct KeyRingValidationSink {
    client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ThrottleKindSink for KeyRingValidationSink {
    fn push(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::PreviewKeyRingRequest {
                    key_ring: Some(keymanagement::KeyRing {
                        name: record.kind,
                        key_ring_id: record.message,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.preview_key_ring(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
}

impl ThrottleKindSink for FlagEvaluationNotificationSink {
    fn push(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .send_flag_evaluation_test_notice(featureflags::SendFlagEvaluationTestNoticeRequest {
                    details: Some(featureflags::send_flag_evaluation_test_notice_request::Details {
                        template_id: record.message,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.tours.v1.TourService`.
struct TourSink {
    client: tours::tour_service_client::TourServiceClient<Channel>,
}

impl ThrottleKindSink for TourSink {
    fn push(&self, record: ThrottleKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_tour(tours::CreateTourRequest {
                    request_id: record.message,
                    tour: Some(tours::Tour {
                        tour_id: record.correlation_id,
                        title: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ThrottleKindLogOnly;

impl ThrottleKindSink for ThrottleKindLogOnly {
    fn push(&self, record: ThrottleKindRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.kind);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_throttle_kind(
    kind: &str,
    record: ThrottleKindRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let key_ring_validation = KeyRingValidationSink {
        client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let flag_evaluation_notification = FlagEvaluationNotificationSink {
        client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(
            channel.clone(),
        ),
    };
    let tour = TourSink {
        client: tours::tour_service_client::TourServiceClient::new(channel),
    };
    let write: &dyn Fn(ThrottleKindRecord) -> Delivery = match kind {
        "key-ring-validation" => &|record| key_ring_validation.push(record),
        "flag-evaluation-notification" => &|record| flag_evaluation_notification.push(record),
        "tour" => &|record| tour.push(record),
        _ => &|record| ThrottleKindLogOnly.push(record),
    };
    write(record).await
}
