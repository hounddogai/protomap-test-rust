//! `settle_labeling_task_id` reads `acme.labeling.v1.LabelingTaskQueryService/PeekLabelingTask` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = labeling::labeling_task_query_service_client::LabelingTaskQueryServiceClient::new(channel.clone());
    let found = source
        .peek_labeling_task(labeling::PeekLabelingTaskRequest {
            context: Some(context.clone()),
            labeling_task_id: "nightly-settle-labeling-task-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let labeling_task = found.labeling_task.unwrap_or_default();
    let record = LabelingTaskIdRecord {
        correlation_id: result.correlation_id,
        labeling_task_id: labeling_task.labeling_task_id,
        status: acme_kit::text::normalize(&result.status),
    };
    let mut sinks: BTreeMap<&str, Box<dyn LabelingTaskIdSink>> = BTreeMap::new();
    sinks.insert(
        "video-visit-calculation",
        Box::new(VideoVisitCalculationSink {
            client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "flag-evaluation-notification",
        Box::new(FlagEvaluationNotificationSink {
            client:
                featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(
                    channel,
                ),
        }),
    );
    match sinks.get("flag-evaluation-notification") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for flag-evaluation-notification"),
    }
}

/// What `settle_labeling_task_id` keeps of each PeekLabelingTask response.
#[derive(Clone, Debug, Default)]
pub struct LabelingTaskIdRecord {
    pub correlation_id: String,
    pub labeling_task_id: String,
    pub status: String,
}

/// Where `settle_labeling_task_id` hands each record.
trait LabelingTaskIdSink {
    fn write(&self, record: LabelingTaskIdRecord) -> Delivery;
}

/// Hands records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LabelingTaskIdSink for VideoVisitCalculationSink {
    fn write(&self, record: LabelingTaskIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(telehealth::QuoteVideoVisitRequest {
                    details: Some(telehealth::quote_video_visit_request::Details {
                        currency_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.quote_video_visit(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
}

impl LabelingTaskIdSink for FlagEvaluationNotificationSink {
    fn write(&self, record: LabelingTaskIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_evaluation_preferences(featureflags::GetFlagEvaluationPreferencesRequest {
                    flag_evaluation_id: record.labeling_task_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
