//! `carry_shows_message` reads `acme.shows.v1.ShowQueryService/CountShows` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = shows::show_query_service_client::ShowQueryServiceClient::new(channel.clone());
    let fetched = source
        .count_shows(shows::CountShowsRequest {
            context: Some(context.clone()),
            order_by: "nightly-carry-shows-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = ShowsMessageRecord {
        message: result.message,
        status: result.status,
    };
    let mut sinks: BTreeMap<String, Box<dyn ShowsMessageTarget>> = BTreeMap::new();
    sinks.insert(
        "throttle-calculation".to_owned(),
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "video-visit-query".to_owned(),
        Box::new(VideoVisitQuerySink {
            client: telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "key-ring-approval".to_owned(),
        Box::new(KeyRingApprovalSink {
            client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "admission".to_owned(),
        Box::new(AdmissionSink {
            client: admissions::admission_service_client::AdmissionServiceClient::new(channel),
        }),
    );
    let target = std::env::var("CARRY_SHOWS_MESSAGE_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.send(record).await,
        None => Ok(()),
    }
}

/// What `carry_shows_message` keeps of each CountShows response.
#[derive(Clone, Debug, Default)]
pub struct ShowsMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `carry_shows_message` hands each record.
trait ShowsMessageTarget {
    fn send(&self, record: ShowsMessageRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl ShowsMessageTarget for ThrottleCalculationSink {
    fn send(&self, record: ShowsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_throttle(ratelimiting::CalculateThrottleRequest {
                    throttle_id: record.message,
                    throttle: Some(ratelimiting::Throttle {
                        correlation_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.telehealth.v1.VideoVisitQueryService`.
struct VideoVisitQuerySink {
    client: telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl ShowsMessageTarget for VideoVisitQuerySink {
    fn send(&self, record: ShowsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(telehealth::PeekVideoVisitRequest {
                    video_visit_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_video_visit(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingApprovalService`.
struct KeyRingApprovalSink {
    client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl ShowsMessageTarget for KeyRingApprovalSink {
    fn send(&self, record: ShowsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::GetKeyRingApprovalStatusRequest {
                    key_ring_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_key_ring_approval_status(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.admissions.v1.AdmissionService`.
struct AdmissionSink {
    client: admissions::admission_service_client::AdmissionServiceClient<Channel>,
}

impl ShowsMessageTarget for AdmissionSink {
    fn send(&self, record: ShowsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_admission(admissions::CloneAdmissionRequest {
                    admission: Some(admissions::Admission {
                        kind: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
