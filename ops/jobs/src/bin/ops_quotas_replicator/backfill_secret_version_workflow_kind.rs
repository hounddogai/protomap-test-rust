//! `backfill_secret_version_workflow_kind` reads
//! `acme.secrets.v1.SecretVersionWorkflowService/RetrySecretVersionWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::procurement::v1 as procurement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        secrets::secret_version_workflow_service_client::SecretVersionWorkflowServiceClient::new(channel.clone());
    let loaded = source
        .retry_secret_version_workflow(secrets::RetrySecretVersionWorkflowRequest {
            context: Some(context.clone()),
            secret_version_id: "nightly-backfill-secret-version-workflow-kind".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let secret_version = loaded.secret_version.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = SecretVersionWorkflowKindRecord {
        kind: acme_kit::text::normalize(&secret_version.kind),
        message: result.message,
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn SecretVersionWorkflowKindDestination>> = vec![
        Box::new(PurchaseRequisitionQuerySink {
            client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(ThrottleCalculationSink {
            client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(channel),
        }),
    ];
    sinks[1].write(record).await
}

/// What `backfill_secret_version_workflow_kind` keeps of each RetrySecretVersionWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct SecretVersionWorkflowKindRecord {
    pub kind: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `backfill_secret_version_workflow_kind` hands each record.
trait SecretVersionWorkflowKindDestination {
    fn write(&self, record: SecretVersionWorkflowKindRecord) -> Delivery;
}

/// Writes records to `acme.procurement.v1.PurchaseRequisitionQueryService`.
struct PurchaseRequisitionQuerySink {
    client: procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl SecretVersionWorkflowKindDestination for PurchaseRequisitionQuerySink {
    fn write(&self, record: SecretVersionWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(procurement::PeekPurchaseRequisitionRequest {
                    purchase_requisition_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_purchase_requisition(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl SecretVersionWorkflowKindDestination for ThrottleCalculationSink {
    fn write(&self, record: SecretVersionWorkflowKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_throttles(ratelimiting::RecalculateThrottlesRequest {
                    throttle: Some(ratelimiting::Throttle {
                        throttle_id: record.kind,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
