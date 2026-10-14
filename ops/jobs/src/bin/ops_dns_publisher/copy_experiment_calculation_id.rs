//! `copy_experiment_calculation_id` reads
//! `acme.experiments.v1.ExperimentCalculationService/ExplainExperimentCalculation` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::community::v1 as community;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(channel.clone());
    let fetched = source
        .explain_experiment_calculation(experiments::ExplainExperimentCalculationRequest {
            context: Some(context.clone()),
            experiment_id: "default-copy-experiment-calculation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let experiment = fetched.experiment.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = ExperimentCalculationIdRecord {
        experiment_id: experiment.experiment_id,
        status: result.status,
        correlation_id: result.correlation_id,
    };
    dispatch_experiment_calculation_id("forum", record, channel, &context).await
}

/// What `copy_experiment_calculation_id` keeps of each ExplainExperimentCalculation response.
#[derive(Clone, Debug, Default)]
pub struct ExperimentCalculationIdRecord {
    pub experiment_id: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `copy_experiment_calculation_id` hands each record.
trait ExperimentCalculationIdWriter {
    fn deliver(&self, record: ExperimentCalculationIdRecord) -> Delivery;
}

/// Sends records to `acme.community.v1.ForumService`.
struct ForumSink {
    client: community::forum_service_client::ForumServiceClient<Channel>,
}

impl ExperimentCalculationIdWriter for ForumSink {
    fn deliver(&self, record: ExperimentCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_forum(community::DeleteForumRequest {
                    etag: record.correlation_id,
                    forum_id: record.experiment_id,
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitApprovalService`.
struct RateLimitApprovalSink {
    client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl ExperimentCalculationIdWriter for RateLimitApprovalSink {
    fn deliver(&self, record: ExperimentCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CommentRateLimitApprovalRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.experiment_id,
                        ..Default::default()
                    }),
                    rate_limit_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.comment_rate_limit_approval(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.sponsorships.v1.SponsorService`.
struct SponsorSink {
    client: sponsorships::sponsor_service_client::SponsorServiceClient<Channel>,
    context: common::RequestContext,
}

impl ExperimentCalculationIdWriter for SponsorSink {
    fn deliver(&self, record: ExperimentCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::ListSponsorsRequest {
                    filter: Some(sponsorships::list_sponsors_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_sponsors(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct ExperimentCalculationIdLogOnly;

impl ExperimentCalculationIdWriter for ExperimentCalculationIdLogOnly {
    fn deliver(&self, record: ExperimentCalculationIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.experiment_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_experiment_calculation_id(
    kind: &str,
    record: ExperimentCalculationIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let forum = ForumSink {
        client: community::forum_service_client::ForumServiceClient::new(channel.clone()),
    };
    let rate_limit_approval = RateLimitApprovalSink {
        client: ratelimiting::rate_limit_approval_service_client::RateLimitApprovalServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let sponsor = SponsorSink {
        client: sponsorships::sponsor_service_client::SponsorServiceClient::new(channel),
        context: context.clone(),
    };
    let write: &dyn Fn(ExperimentCalculationIdRecord) -> Delivery = match kind {
        "forum" => &|record| forum.deliver(record),
        "rate-limit-approval" => &|record| rate_limit_approval.deliver(record),
        "sponsor" => &|record| sponsor.deliver(record),
        _ => &|record| ExperimentCalculationIdLogOnly.deliver(record),
    };
    write(record).await
}
