//! `hand_off_key_ring_workflow_id` reads `acme.keymanagement.v1.KeyRingWorkflowService/CancelKeyRingWorkflow`
//! and hands what it keeps to `acme.anomalies.v1.AnomalyQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel.clone());
    let loaded = source
        .cancel_key_ring_workflow(keymanagement::CancelKeyRingWorkflowRequest {
            context: Some(context.clone()),
            key_ring_id: "current-hand-off-key-ring-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let key_ring = loaded.key_ring.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = OffKeyRingWorkflowIdRecord {
        correlation_id: result.correlation_id,
        key_ring_id: key_ring.key_ring_id,
        status: result.status,
        reference_id: outcome.reference_id,
    };
    AnomalyQuerySink {
        client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel),
    }
    .apply(record)
    .await
}

/// What `hand_off_key_ring_workflow_id` keeps of each CancelKeyRingWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct OffKeyRingWorkflowIdRecord {
    pub correlation_id: String,
    pub key_ring_id: String,
    pub status: String,
    pub reference_id: String,
}

/// Where `hand_off_key_ring_workflow_id` hands each record.
trait OffKeyRingWorkflowIdWriter {
    fn apply(&self, record: OffKeyRingWorkflowIdRecord) -> Delivery;
}

/// Writes records to `acme.anomalies.v1.AnomalyQueryService`.
struct AnomalyQuerySink {
    client: anomalies::anomaly_query_service_client::AnomalyQueryServiceClient<Channel>,
}

impl OffKeyRingWorkflowIdWriter for AnomalyQuerySink {
    fn apply(&self, record: OffKeyRingWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_anomalies(anomalies::CountAnomaliesRequest {
                    page: Some(common::PageRequest {
                        page_token: record.key_ring_id,
                        ..Default::default()
                    }),
                    filter: Some(anomalies::count_anomalies_request::Filter {
                        query: record.reference_id,
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
