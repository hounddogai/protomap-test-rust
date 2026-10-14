//! `forward_documents_id` reads `acme.documents.v1.DocumentQueryService/SummarizeDocuments` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::promotions::v1 as promotions;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = documents::document_query_service_client::DocumentQueryServiceClient::new(channel.clone());
    let loaded = source
        .summarize_documents(documents::SummarizeDocumentsRequest {
            context: Some(context.clone()),
            reason: "default-forward-documents-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let period = loaded.period.unwrap_or_default();
    let record = DocumentsIdRecord {
        correlation_id: result.correlation_id,
        timezone: period.timezone,
        status: result.status,
        report_id: acme_kit::text::normalize(&loaded.report_id),
    };
    act_on_documents_id("flag-rollout-notification", record, channel, context).await
}

/// What `forward_documents_id` keeps of each SummarizeDocuments response.
#[derive(Clone, Debug, Default)]
pub struct DocumentsIdRecord {
    pub correlation_id: String,
    pub timezone: String,
    pub status: String,
    pub report_id: String,
}

/// Where `forward_documents_id` hands each record.
trait DocumentsIdDestination {
    fn write(&self, record: DocumentsIdRecord) -> Delivery;
}

/// Sends records to `acme.promotions.v1.PromotionService`.
struct PromotionSink {
    client: promotions::promotion_service_client::PromotionServiceClient<Channel>,
}

impl DocumentsIdDestination for PromotionSink {
    fn write(&self, record: DocumentsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_promotion(promotions::ClonePromotionRequest {
                    promotion: Some(promotions::Promotion {
                        description: record.correlation_id,
                        name: record.status,
                        ..Default::default()
                    }),
                    request_id: record.report_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagRolloutNotificationService`.
struct FlagRolloutNotificationSink {
    client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl DocumentsIdDestination for FlagRolloutNotificationSink {
    fn write(&self, record: DocumentsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::SendFlagRolloutTestNoticeRequest {
                    details: Some(featureflags::send_flag_rollout_test_notice_request::Details {
                        template_id: record.status,
                        ..Default::default()
                    }),
                    flag_rollout_id: record.timezone,
                    ..Default::default()
                }),
                &context,
            )?;
            client.send_flag_rollout_test_notice(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
}

impl DocumentsIdDestination for FlagEvaluationNotificationSink {
    fn write(&self, record: DocumentsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .unsubscribe_flag_evaluation_updates(featureflags::UnsubscribeFlagEvaluationUpdatesRequest {
                    reason: record.correlation_id,
                    details: Some(featureflags::unsubscribe_flag_evaluation_updates_request::Details {
                        channel: record.status,
                        template_id: record.timezone,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.admissions.v1.AdmissionQueryService`.
struct AdmissionQuerySink {
    client: admissions::admission_query_service_client::AdmissionQueryServiceClient<Channel>,
}

impl DocumentsIdDestination for AdmissionQuerySink {
    fn write(&self, record: DocumentsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_admission(admissions::LookupAdmissionRequest {
                    admission_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_documents_id(
    kind: &str,
    record: DocumentsIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(DocumentsIdRecord) -> Delivery>> = BTreeMap::new();
    let promotion_channel = channel.clone();
    actions.insert(
        "promotion",
        Box::new(move |record| {
            PromotionSink {
                client: promotions::promotion_service_client::PromotionServiceClient::new(promotion_channel.clone()),
            }
            .write(record)
        }),
    );
    let flag_rollout_notification_channel = channel.clone();
    let flag_rollout_notification_context = context.clone();
    actions.insert(
        "flag-rollout-notification",
        Box::new(move |record| {
            FlagRolloutNotificationSink {
                client:
                    featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient::new(
                        flag_rollout_notification_channel.clone(),
                    ),
                context: flag_rollout_notification_context.clone(),
            }
            .write(record)
        }),
    );
    let flag_evaluation_notification_channel = channel.clone();
    actions.insert("flag-evaluation-notification", Box::new(move |record| FlagEvaluationNotificationSink { client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(flag_evaluation_notification_channel.clone()) }.write(record)));
    let admission_query_channel = channel.clone();
    actions.insert(
        "admission-query",
        Box::new(move |record| {
            AdmissionQuerySink {
                client: admissions::admission_query_service_client::AdmissionQueryServiceClient::new(
                    admission_query_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
