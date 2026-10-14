//! `push_quota_bucket_decision_id` reads
//! `acme.ratelimiting.v1.QuotaBucketValidationService/ExplainQuotaBucketDecision` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::personalization::v1 as personalization;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(channel.clone());
    let fetched = source
        .explain_quota_bucket_decision(ratelimiting::ExplainQuotaBucketDecisionRequest {
            context: Some(context.clone()),
            reason: "nightly-push-quota-bucket-decision-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let quota_bucket = fetched.quota_bucket.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = QuotaBucketDecisionIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        currency_code: quota_bucket.currency_code,
        status: outcome.status,
    };
    act_on_quota_bucket_decision_id("config-change-approval", record, channel, context).await
}

/// What `push_quota_bucket_decision_id` keeps of each ExplainQuotaBucketDecision response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketDecisionIdRecord {
    pub correlation_id: String,
    pub currency_code: String,
    pub status: String,
}

/// Where `push_quota_bucket_decision_id` hands each record.
trait QuotaBucketDecisionIdDestination {
    fn apply(&self, record: QuotaBucketDecisionIdRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigChangeApprovalService`.
struct ConfigChangeApprovalSink {
    client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient<Channel>,
}

impl QuotaBucketDecisionIdDestination for ConfigChangeApprovalSink {
    fn apply(&self, record: QuotaBucketDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delegate_config_change_approval(configuration::DelegateConfigChangeApprovalRequest {
                    reason: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.personalization.v1.ProfileCalculationService`.
struct ProfileCalculationSink {
    client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl QuotaBucketDecisionIdDestination for ProfileCalculationSink {
    fn apply(&self, record: QuotaBucketDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(personalization::ExplainProfileCalculationRequest {
                    details: Some(personalization::explain_profile_calculation_request::Details {
                        locale: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    profile: Some(personalization::Profile {
                        profile_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_profile_calculation(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
}

impl QuotaBucketDecisionIdDestination for VideoVisitCalculationSink {
    fn apply(&self, record: QuotaBucketDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_video_visit(telehealth::EstimateVideoVisitRequest {
                    details: Some(telehealth::estimate_video_visit_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl QuotaBucketDecisionIdDestination for ForumSink {
    fn apply(&self, record: QuotaBucketDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_forums(community::ListForumsRequest {
                    filter: Some(community::list_forums_request::Filter {
                        owner_id: record.correlation_id,
                        query: record.currency_code,
                        region: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_quota_bucket_decision_id(
    kind: &str,
    record: QuotaBucketDecisionIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(QuotaBucketDecisionIdRecord) -> Delivery>> = BTreeMap::new();
    let config_change_approval_channel = channel.clone();
    actions.insert(
        "config-change-approval",
        Box::new(move |record| {
            ConfigChangeApprovalSink {
                client: configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(
                    config_change_approval_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let profile_calculation_channel = channel.clone();
    let profile_calculation_context = context.clone();
    actions.insert(
        "profile-calculation",
        Box::new(move |record| {
            ProfileCalculationSink {
                client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient::new(
                    profile_calculation_channel.clone(),
                ),
                context: profile_calculation_context.clone(),
            }
            .apply(record)
        }),
    );
    let video_visit_calculation_channel = channel.clone();
    actions.insert(
        "video-visit-calculation",
        Box::new(move |record| {
            VideoVisitCalculationSink {
                client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
                    video_visit_calculation_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let forum_channel = channel.clone();
    actions.insert(
        "forum",
        Box::new(move |record| {
            ForumSink {
                client: community::forum_service_client::ForumServiceClient::new(forum_channel.clone()),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
