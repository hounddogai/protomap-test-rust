//! `hand_off_flag_rollout_versions_message` reads
//! `acme.featureflags.v1.FlagRolloutHistoryService/ListFlagRolloutVersions` and hands what it keeps to
//! `acme.broadcasts.v1.BroadcastQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_rollout_history_service_client::FlagRolloutHistoryServiceClient::new(channel.clone());
    let loaded = source
        .list_flag_rollout_versions(featureflags::ListFlagRolloutVersionsRequest {
            context: Some(context.clone()),
            order_by: "current-hand-off-flag-rollout-versions-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = OffFlagRolloutVersionsMessageRecord {
        message: result.message,
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    BroadcastQuerySink {
        client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel),
    }
    .write(record)
    .await
}

/// What `hand_off_flag_rollout_versions_message` keeps of each ListFlagRolloutVersions response.
#[derive(Clone, Debug, Default)]
pub struct OffFlagRolloutVersionsMessageRecord {
    pub message: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `hand_off_flag_rollout_versions_message` hands each record.
trait OffFlagRolloutVersionsMessageDestination {
    fn write(&self, record: OffFlagRolloutVersionsMessageRecord) -> Delivery;
}

/// Sends records to `acme.broadcasts.v1.BroadcastQueryService`.
struct BroadcastQuerySink {
    client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient<Channel>,
}

impl OffFlagRolloutVersionsMessageDestination for BroadcastQuerySink {
    fn write(&self, record: OffFlagRolloutVersionsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_broadcast(broadcasts::LookupBroadcastRequest {
                    broadcast_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
