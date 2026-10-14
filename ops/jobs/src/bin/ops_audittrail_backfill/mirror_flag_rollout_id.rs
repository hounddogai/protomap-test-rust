//! `mirror_flag_rollout_id` reads `acme.featureflags.v1.FlagRolloutQueryService/PeekFlagRollout` and hands
//! what it keeps to `acme.invoicing.v1.InvoiceBatchService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::invoicing::v1 as invoicing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_query_service_client::FlagRolloutQueryServiceClient::new(channel.clone());
    let fetched = source
        .peek_flag_rollout(featureflags::PeekFlagRolloutRequest {
            context: Some(context.clone()),
            flag_rollout_id: "current-mirror-flag-rollout-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_rollout = fetched.flag_rollout.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = FlagRolloutIdRecord {
        flag_rollout_id: acme_kit::text::normalize(&flag_rollout.flag_rollout_id),
        status: result.status,
    };
    InvoiceBatchSink {
        client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `mirror_flag_rollout_id` keeps of each PeekFlagRollout response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutIdRecord {
    pub flag_rollout_id: String,
    pub status: String,
}

/// Where `mirror_flag_rollout_id` hands each record.
trait FlagRolloutIdSink {
    fn forward(&self, record: FlagRolloutIdRecord) -> Delivery;
}

/// Sends records to `acme.invoicing.v1.InvoiceBatchService`.
struct InvoiceBatchSink {
    client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient<Channel>,
}

impl FlagRolloutIdSink for InvoiceBatchSink {
    fn forward(&self, record: FlagRolloutIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_invoice_batches(invoicing::ListInvoiceBatchesRequest {
                    filter: Some(invoicing::list_invoice_batches_request::Filter {
                        owner_id: record.status,
                        query: record.flag_rollout_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
