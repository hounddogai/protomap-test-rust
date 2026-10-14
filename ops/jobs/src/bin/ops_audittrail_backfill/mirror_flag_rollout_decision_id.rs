//! `mirror_flag_rollout_decision_id` reads
//! `acme.featureflags.v1.FlagRolloutValidationService/ExplainFlagRolloutDecision` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_validation_service_client::FlagRolloutValidationServiceClient::new(channel.clone());
    let reply = source
        .explain_flag_rollout_decision(featureflags::ExplainFlagRolloutDecisionRequest {
            context: Some(context.clone()),
            reason: "primary-mirror-flag-rollout-decision-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_rollout = reply.flag_rollout.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = FlagRolloutDecisionIdRecord {
        flag_rollout_id: acme_kit::text::normalize(&flag_rollout.flag_rollout_id),
        status: outcome.status,
    };
    let sinks: Vec<(&str, Box<dyn FlagRolloutDecisionIdDestination>)> = vec![
        (
            "fan-session-query",
            Box::new(FanSessionQuerySink {
                client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "flag-rollout-admin",
            Box::new(FlagRolloutAdminSink {
                client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-rollout-admin" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `mirror_flag_rollout_decision_id` keeps of each ExplainFlagRolloutDecision response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutDecisionIdRecord {
    pub flag_rollout_id: String,
    pub status: String,
}

/// Where `mirror_flag_rollout_decision_id` hands each record.
trait FlagRolloutDecisionIdDestination {
    fn deliver(&self, record: FlagRolloutDecisionIdRecord) -> Delivery;
}

/// Passes records to `acme.fanportal.v1.FanSessionQueryService`.
struct FanSessionQuerySink {
    client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient<Channel>,
}

impl FlagRolloutDecisionIdDestination for FanSessionQuerySink {
    fn deliver(&self, record: FlagRolloutDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_fan_sessions(fanportal::FindFanSessionsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.flag_rollout_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagRolloutAdminService`.
struct FlagRolloutAdminSink {
    client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient<Channel>,
}

impl FlagRolloutDecisionIdDestination for FlagRolloutAdminSink {
    fn deliver(&self, record: FlagRolloutDecisionIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .purge_flag_rollout(featureflags::PurgeFlagRolloutRequest {
                    etag: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
