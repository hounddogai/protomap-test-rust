//! `route_video_visit_title` reads `acme.telehealth.v1.VideoVisitQueryService/PeekVideoVisit` and hands what
//! it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = telehealth::video_visit_query_service_client::VideoVisitQueryServiceClient::new(channel.clone());
    let fetched = source
        .peek_video_visit(telehealth::PeekVideoVisitRequest {
            context: Some(context.clone()),
            video_visit_id: "nightly-route-video-visit-title".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let video_visit = fetched.video_visit.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = VideoVisitTitleRecord {
        title: acme_kit::text::normalize(&video_visit.title),
        video_visit_id: video_visit.video_visit_id,
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
    };
    deliver_video_visit_title("box-office-sale-query", record, channel, &context).await
}

/// What `route_video_visit_title` keeps of each PeekVideoVisit response.
#[derive(Clone, Debug, Default)]
pub struct VideoVisitTitleRecord {
    pub title: String,
    pub video_visit_id: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `route_video_visit_title` hands each record.
trait VideoVisitTitleDestination {
    fn apply(&self, record: VideoVisitTitleRecord) -> Delivery;
}

/// Passes records to `acme.authorization.v1.AccessPolicyHistoryService`.
struct AccessPolicyHistorySink {
    client: authorization::access_policy_history_service_client::AccessPolicyHistoryServiceClient<Channel>,
}

impl VideoVisitTitleDestination for AccessPolicyHistorySink {
    fn apply(&self, record: VideoVisitTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .prune_access_policy_versions(authorization::PruneAccessPolicyVersionsRequest {
                    access_policy_id: record.video_visit_id,
                    reason: record.status,
                    etag: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl VideoVisitTitleDestination for FanSessionSink {
    fn apply(&self, record: VideoVisitTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_fan_session(fanportal::DeleteFanSessionRequest {
                    reason: record.status,
                    fan_session_id: record.video_visit_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
}

impl VideoVisitTitleDestination for BoxOfficeSaleQuerySink {
    fn apply(&self, record: VideoVisitTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_box_office_sales(boxoffice::BrowseBoxOfficeSalesRequest {
                    filter: Some(boxoffice::browse_box_office_sales_request::Filter {
                        query: record.video_visit_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationCalculationService`.
struct FlagEvaluationCalculationSink {
    client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl VideoVisitTitleDestination for FlagEvaluationCalculationSink {
    fn apply(&self, record: VideoVisitTitleRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::CalculateFlagEvaluationRequest {
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        kind: record.title,
                        ..Default::default()
                    }),
                    flag_evaluation_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_flag_evaluation(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct VideoVisitTitleLogOnly;

impl VideoVisitTitleDestination for VideoVisitTitleLogOnly {
    fn apply(&self, record: VideoVisitTitleRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.title);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_video_visit_title(
    kind: &str,
    record: VideoVisitTitleRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn VideoVisitTitleDestination> = match kind {
        "access-policy-history" => Box::new(AccessPolicyHistorySink {
            client: authorization::access_policy_history_service_client::AccessPolicyHistoryServiceClient::new(
                channel.clone(),
            ),
        }),
        "fan-session" => Box::new(FanSessionSink {
            client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
        }),
        "box-office-sale-query" => Box::new(BoxOfficeSaleQuerySink {
            client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        "flag-evaluation-calculation" => Box::new(FlagEvaluationCalculationSink {
            client:
                featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient::new(
                    channel,
                ),
            context: context.clone(),
        }),
        _ => Box::new(VideoVisitTitleLogOnly),
    };
    sink.apply(record).await
}
