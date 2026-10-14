//! `relay_purchase_requisitions_status` reads
//! `acme.procurement.v1.PurchaseRequisitionQueryService/CountPurchaseRequisitions` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::procurement::v1 as procurement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = procurement::purchase_requisition_query_service_client::PurchaseRequisitionQueryServiceClient::new(
        channel.clone(),
    );
    let fetched = source
        .count_purchase_requisitions(procurement::CountPurchaseRequisitionsRequest {
            context: Some(context.clone()),
            order_by: "current-relay-purchase-requisitions-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = PurchaseRequisitionsStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    dispatch_purchase_requisitions_status("key-rotation-workflow", record, channel, &context).await
}

/// What `relay_purchase_requisitions_status` keeps of each CountPurchaseRequisitions response.
#[derive(Clone, Debug, Default)]
pub struct PurchaseRequisitionsStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `relay_purchase_requisitions_status` hands each record.
trait PurchaseRequisitionsStatusTarget {
    fn deliver(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl PurchaseRequisitionsStatusTarget for KeyRotationWorkflowSink {
    fn deliver(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_key_rotation_workflows(keymanagement::ListKeyRotationWorkflowsRequest {
                    filter: Some(keymanagement::list_key_rotation_workflows_request::Filter {
                        owner_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateWorkflowService`.
struct CertificateWorkflowSink {
    client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient<Channel>,
}

impl PurchaseRequisitionsStatusTarget for CertificateWorkflowSink {
    fn deliver(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .inspect_certificate_workflow(certificates::InspectCertificateWorkflowRequest {
                    certificate_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct PurchaseRequisitionsStatusLogOnly;

impl PurchaseRequisitionsStatusTarget for PurchaseRequisitionsStatusLogOnly {
    fn deliver(&self, record: PurchaseRequisitionsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_purchase_requisitions_status(
    kind: &str,
    record: PurchaseRequisitionsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let key_rotation_workflow = KeyRotationWorkflowSink {
        client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
            channel.clone(),
        ),
    };
    let certificate_workflow = CertificateWorkflowSink {
        client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient::new(channel),
    };
    let write: &dyn Fn(PurchaseRequisitionsStatusRecord) -> Delivery = match kind {
        "key-rotation-workflow" => &|record| key_rotation_workflow.deliver(record),
        "certificate-workflow" => &|record| certificate_workflow.deliver(record),
        _ => &|record| PurchaseRequisitionsStatusLogOnly.deliver(record),
    };
    write(record).await
}
