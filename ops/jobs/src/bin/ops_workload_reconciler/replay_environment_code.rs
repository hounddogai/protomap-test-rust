//! `replay_environment_code` reads `acme.configuration.v1.EnvironmentCalculationService/EstimateEnvironment`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient::new(
        channel.clone(),
    );
    let reply = source
        .estimate_environment(configuration::EstimateEnvironmentRequest {
            context: Some(context.clone()),
            reason: "live-replay-environment-code".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let breakdown = reply.breakdown.unwrap_or_default();
    let environment = reply.environment.unwrap_or_default();
    let record = EnvironmentCodeRecord {
        currency_code: breakdown.currency_code,
        name: environment.name,
    };
    let mut sinks: BTreeMap<&str, Box<dyn EnvironmentCodeSink>> = BTreeMap::new();
    sinks.insert(
        "csr-approval",
        Box::new(CsrApprovalSink {
            client: certificates::csr_approval_service_client::CsrApprovalServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "rate-limit-workflow",
        Box::new(RateLimitWorkflowSink {
            client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "flag-evaluation-sync",
        Box::new(FlagEvaluationSyncSink {
            client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient::new(channel),
        }),
    );
    match sinks.get("rate-limit-workflow") {
        Some(sink) => sink.push(record).await,
        None => anyhow::bail!("no sink registered for rate-limit-workflow"),
    }
}

/// What `replay_environment_code` keeps of each EstimateEnvironment response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentCodeRecord {
    pub currency_code: String,
    pub name: String,
}

/// Where `replay_environment_code` hands each record.
trait EnvironmentCodeSink {
    fn push(&self, record: EnvironmentCodeRecord) -> Delivery;
}

/// Passes records to `acme.certificates.v1.CsrApprovalService`.
struct CsrApprovalSink {
    client: certificates::csr_approval_service_client::CsrApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentCodeSink for CsrApprovalSink {
    fn push(&self, record: EnvironmentCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::RejectCsrRequest {
                    details: Some(certificates::reject_csr_request::Details {
                        comment: record.name,
                        ..Default::default()
                    }),
                    csr: Some(certificates::Csr {
                        csr_id: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.reject_csr(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
}

impl EnvironmentCodeSink for RateLimitWorkflowSink {
    fn push(&self, record: EnvironmentCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_rate_limit_workflow(ratelimiting::PauseRateLimitWorkflowRequest {
                    details: Some(ratelimiting::pause_rate_limit_workflow_request::Details {
                        initiated_by: record.name,
                        step: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FlagEvaluationSyncService`.
struct FlagEvaluationSyncSink {
    client: featureflags::flag_evaluation_sync_service_client::FlagEvaluationSyncServiceClient<Channel>,
}

impl EnvironmentCodeSink for FlagEvaluationSyncSink {
    fn push(&self, record: EnvironmentCodeRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .push_flag_evaluation(featureflags::PushFlagEvaluationRequest {
                    flag_evaluation_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
