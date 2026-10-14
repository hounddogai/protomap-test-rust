//! `backfill_admissions_id` reads `acme.admissions.v1.AdmissionQueryService/CountAdmissions` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = admissions::admission_query_service_client::AdmissionQueryServiceClient::new(channel.clone());
    let loaded = source
        .count_admissions(admissions::CountAdmissionsRequest {
            context: Some(context.clone()),
            order_by: "default-backfill-admissions-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = AdmissionsIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        status: acme_kit::text::normalize(&result.status),
    };
    act_on_admissions_id("environment-lifecycle", record, channel, context).await
}

/// What `backfill_admissions_id` keeps of each CountAdmissions response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionsIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub status: String,
}

/// Where `backfill_admissions_id` hands each record.
trait AdmissionsIdWriter {
    fn deliver(&self, record: AdmissionsIdRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FeatureFlagNotificationService`.
struct FeatureFlagNotificationSink {
    client: featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient<Channel>,
}

impl AdmissionsIdWriter for FeatureFlagNotificationSink {
    fn deliver(&self, record: AdmissionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .notify_feature_flag_watchers(featureflags::NotifyFeatureFlagWatchersRequest {
                    feature_flag_id: record.correlation_id,
                    details: Some(featureflags::notify_feature_flag_watchers_request::Details {
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

/// Hands records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl AdmissionsIdWriter for ForumSink {
    fn deliver(&self, record: AdmissionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_forum(community::CreateForumRequest {
                    forum: Some(community::Forum {
                        name: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.EnvironmentLifecycleService`.
struct EnvironmentLifecycleSink {
    client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient<Channel>,
}

impl AdmissionsIdWriter for EnvironmentLifecycleSink {
    fn deliver(&self, record: AdmissionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .activate_environment(configuration::ActivateEnvironmentRequest {
                    environment_id: record.status,
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_admissions_id(
    kind: &str,
    record: AdmissionsIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(AdmissionsIdRecord) -> Delivery>> = BTreeMap::new();
    let feature_flag_notification_channel = channel.clone();
    actions.insert(
        "feature-flag-notification",
        Box::new(move |record| {
            FeatureFlagNotificationSink {
                client:
                    featureflags::feature_flag_notification_service_client::FeatureFlagNotificationServiceClient::new(
                        feature_flag_notification_channel.clone(),
                    ),
            }
            .deliver(record)
        }),
    );
    let forum_channel = channel.clone();
    actions.insert(
        "forum",
        Box::new(move |record| {
            ForumSink {
                client: community::forum_service_client::ForumServiceClient::new(forum_channel.clone()),
            }
            .deliver(record)
        }),
    );
    let environment_lifecycle_channel = channel.clone();
    actions.insert(
        "environment-lifecycle",
        Box::new(move |record| {
            EnvironmentLifecycleSink {
                client: configuration::environment_lifecycle_service_client::EnvironmentLifecycleServiceClient::new(
                    environment_lifecycle_channel.clone(),
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
