//! `replay_csr_message` reads `acme.certificates.v1.CsrLifecycleService/TerminateCsr` and hands what it keeps
//! to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::metrics::v1 as metrics;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient::new(channel.clone());
    let fetched = source
        .terminate_csr(certificates::TerminateCsrRequest {
            context: Some(context.clone()),
            csr_id: "main-replay-csr-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = CsrMessageRecord {
        message: result.message,
        reference_id: outcome.reference_id,
        status: acme_kit::text::normalize(&result.status),
    };
    let mut sinks: BTreeMap<&str, Box<dyn CsrMessageWriter>> = BTreeMap::new();
    sinks.insert(
        "key-ring-workflow",
        Box::new(KeyRingWorkflowSink {
            client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "invoice-batch",
        Box::new(InvoiceBatchSink {
            client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "metric-series-query",
        Box::new(MetricSeriesQuerySink {
            client: metrics::metric_series_query_service_client::MetricSeriesQueryServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("invoice-batch") else {
        anyhow::bail!("no sink registered for invoice-batch");
    };
    sink.apply(record).await
}

/// What `replay_csr_message` keeps of each TerminateCsr response.
#[derive(Clone, Debug, Default)]
pub struct CsrMessageRecord {
    pub message: String,
    pub reference_id: String,
    pub status: String,
}

/// Where `replay_csr_message` hands each record.
trait CsrMessageWriter {
    fn apply(&self, record: CsrMessageRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRingWorkflowService`.
struct KeyRingWorkflowSink {
    client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient<Channel>,
}

impl CsrMessageWriter for KeyRingWorkflowSink {
    fn apply(&self, record: CsrMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .inspect_key_ring_workflow(keymanagement::InspectKeyRingWorkflowRequest {
                    key_ring_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.invoicing.v1.InvoiceBatchService`.
struct InvoiceBatchSink {
    client: invoicing::invoice_batch_service_client::InvoiceBatchServiceClient<Channel>,
    context: common::RequestContext,
}

impl CsrMessageWriter for InvoiceBatchSink {
    fn apply(&self, record: CsrMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(invoicing::DeleteInvoiceBatchRequest {
                    etag: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_invoice_batch(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.metrics.v1.MetricSeriesQueryService`.
struct MetricSeriesQuerySink {
    client: metrics::metric_series_query_service_client::MetricSeriesQueryServiceClient<Channel>,
}

impl CsrMessageWriter for MetricSeriesQuerySink {
    fn apply(&self, record: CsrMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_metric_series(metrics::LookupMetricSeriesRequest {
                    metric_series_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
