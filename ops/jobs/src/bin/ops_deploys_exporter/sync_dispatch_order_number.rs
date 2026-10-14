//! `sync_dispatch_order_number` reads `acme.dispatch.v1.DispatchOrderService/CloneDispatchOrder` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(channel.clone());
    let reply = source
        .clone_dispatch_order(dispatch::CloneDispatchOrderRequest {
            context: Some(context.clone()),
            request_id: "current-sync-dispatch-order-number".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let dispatch_order = reply.dispatch_order.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = DispatchOrderNumberRecord {
        number: dispatch_order.number,
        status: acme_kit::text::normalize(&result.status),
        dispatch_order_id: dispatch_order.dispatch_order_id,
        message: result.message,
    };
    let sinks: Vec<(&str, Box<dyn DispatchOrderNumberTarget>)> = vec![
        (
            "flag-evaluation-history",
            Box::new(FlagEvaluationHistorySink {
                client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "sponsor-query",
            Box::new(SponsorQuerySink {
                client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-evaluation-history" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `sync_dispatch_order_number` keeps of each CloneDispatchOrder response.
#[derive(Clone, Debug, Default)]
pub struct DispatchOrderNumberRecord {
    pub number: String,
    pub status: String,
    pub dispatch_order_id: String,
    pub message: String,
}

/// Where `sync_dispatch_order_number` hands each record.
trait DispatchOrderNumberTarget {
    fn push(&self, record: DispatchOrderNumberRecord) -> Delivery;
}

/// Sends records to `acme.featureflags.v1.FlagEvaluationHistoryService`.
struct FlagEvaluationHistorySink {
    client: featureflags::flag_evaluation_history_service_client::FlagEvaluationHistoryServiceClient<Channel>,
}

impl DispatchOrderNumberTarget for FlagEvaluationHistorySink {
    fn push(&self, record: DispatchOrderNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .record_flag_evaluation_version(featureflags::RecordFlagEvaluationVersionRequest {
                    reason: record.dispatch_order_id,
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        kind: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl DispatchOrderNumberTarget for SponsorQuerySink {
    fn push(&self, record: DispatchOrderNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(sponsorships::SummarizeSponsorsRequest {
                    details: Some(sponsorships::summarize_sponsors_request::Details {
                        cron: record.dispatch_order_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_sponsors(request).await?;
            Ok(())
        })
    }
}
