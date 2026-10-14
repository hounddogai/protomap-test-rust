//! `replay_broadcasts_token` reads `acme.broadcasts.v1.BroadcastService/ListBroadcasts` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = broadcasts::broadcast_service_client::BroadcastServiceClient::new(channel.clone());
    let reply = source
        .list_broadcasts(broadcasts::ListBroadcastsRequest {
            context: Some(context.clone()),
            order_by: "current-replay-broadcasts-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = reply.page_info.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = BroadcastsTokenRecord {
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        status: acme_kit::text::normalize(&result.status),
        correlation_id: result.correlation_id,
    };
    deliver_broadcasts_token("admission", record, channel, &context).await
}

/// What `replay_broadcasts_token` keeps of each ListBroadcasts response.
#[derive(Clone, Debug, Default)]
pub struct BroadcastsTokenRecord {
    pub next_page_token: String,
    pub status: String,
    pub correlation_id: String,
}

/// Where `replay_broadcasts_token` hands each record.
trait BroadcastsTokenTarget {
    fn forward(&self, record: BroadcastsTokenRecord) -> Delivery;
}

/// Sends records to `acme.admissions.v1.AdmissionService`.
struct AdmissionSink {
    client: admissions::admission_service_client::AdmissionServiceClient<Channel>,
    context: common::RequestContext,
}

impl BroadcastsTokenTarget for AdmissionSink {
    fn forward(&self, record: BroadcastsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::CloneAdmissionRequest {
                    admission: Some(admissions::Admission {
                        correlation_id: record.next_page_token,
                        admission_id: record.status,
                        kind: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.clone_admission(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl BroadcastsTokenTarget for FlagRolloutCalculationSink {
    fn forward(&self, record: BroadcastsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::SimulateFlagRolloutRequest {
                    flag_rollout_id: record.status,
                    reason: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_flag_rollout(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.shows.v1.ShowCalculationService`.
struct ShowCalculationSink {
    client: shows::show_calculation_service_client::ShowCalculationServiceClient<Channel>,
}

impl BroadcastsTokenTarget for ShowCalculationSink {
    fn forward(&self, record: BroadcastsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_shows(shows::RecalculateShowsRequest {
                    details: Some(shows::recalculate_shows_request::Details {
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

/// Passes records to `acme.invoicing.v1.InvoiceBatchService`.
struct InvoiceBatchSink {
    client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient<Channel>,
    context: common::RequestContext,
}

impl BroadcastsTokenTarget for InvoiceBatchSink {
    fn forward(&self, record: BroadcastsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(invoicing::ListInvoiceBatchesRequest {
                    filter: Some(invoicing::list_invoice_batches_request::Filter {
                        query: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_invoice_batches(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct BroadcastsTokenLogOnly;

impl BroadcastsTokenTarget for BroadcastsTokenLogOnly {
    fn forward(&self, record: BroadcastsTokenRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.next_page_token);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_broadcasts_token(
    kind: &str,
    record: BroadcastsTokenRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn BroadcastsTokenTarget> = match kind {
        "admission" => Box::new(AdmissionSink {
            client: admissions::admission_service_client::AdmissionServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "flag-rollout-calculation" => Box::new(FlagRolloutCalculationSink {
            client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "show-calculation" => Box::new(ShowCalculationSink {
            client: shows::show_calculation_service_client::ShowCalculationServiceClient::new(channel.clone()),
        }),
        "invoice-batch" => Box::new(InvoiceBatchSink {
            client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(BroadcastsTokenLogOnly),
    };
    sink.forward(record).await
}
