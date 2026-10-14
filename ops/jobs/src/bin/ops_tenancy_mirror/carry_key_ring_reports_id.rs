//! `carry_key_ring_reports_id` reads `acme.keymanagement.v1.KeyRingReportingService/ListKeyRingReports` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::dispatch::v1 as dispatch;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_reporting_service_client::KeyRingReportingServiceClient::new(channel.clone());
    let reply = source
        .list_key_ring_reports(keymanagement::ListKeyRingReportsRequest {
            context: Some(context.clone()),
            order_by: "default-carry-key-ring-reports-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = KeyRingReportsIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
    };
    act_on_key_ring_reports_id("throttle-workflow", record, channel, context).await
}

/// What `carry_key_ring_reports_id` keeps of each ListKeyRingReports response.
#[derive(Clone, Debug, Default)]
pub struct KeyRingReportsIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
}

/// Where `carry_key_ring_reports_id` hands each record.
trait KeyRingReportsIdWriter {
    fn write(&self, record: KeyRingReportsIdRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRotationApprovalService`.
struct KeyRotationApprovalSink {
    client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient<Channel>,
}

impl KeyRingReportsIdWriter for KeyRotationApprovalSink {
    fn write(&self, record: KeyRingReportsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_key_rotation_approval_status(keymanagement::GetKeyRotationApprovalStatusRequest {
                    key_rotation_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.dispatch.v1.DispatchOrderService`.
struct DispatchOrderSink {
    client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient<Channel>,
}

impl KeyRingReportsIdWriter for DispatchOrderSink {
    fn write(&self, record: KeyRingReportsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_dispatch_order(dispatch::CloneDispatchOrderRequest {
                    dispatch_order: Some(dispatch::DispatchOrder {
                        number: record.correlation_id,
                        dispatch_order_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.ThrottleWorkflowService`.
struct ThrottleWorkflowSink {
    client: ratelimiting::throttle_workflow_service_client::ThrottleWorkflowServiceClient<Channel>,
}

impl KeyRingReportsIdWriter for ThrottleWorkflowSink {
    fn write(&self, record: KeyRingReportsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_throttle_workflows(ratelimiting::ListThrottleWorkflowsRequest {
                    filter: Some(ratelimiting::list_throttle_workflows_request::Filter {
                        region: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_key_ring_reports_id(
    kind: &str,
    record: KeyRingReportsIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(KeyRingReportsIdRecord) -> Delivery>> = BTreeMap::new();
    let key_rotation_approval_channel = channel.clone();
    actions.insert(
        "key-rotation-approval",
        Box::new(move |record| {
            KeyRotationApprovalSink {
                client: keymanagement::key_rotation_approval_service_client::KeyRotationApprovalServiceClient::new(
                    key_rotation_approval_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let dispatch_order_channel = channel.clone();
    actions.insert(
        "dispatch-order",
        Box::new(move |record| {
            DispatchOrderSink {
                client: dispatch::dispatch_order_service_client::DispatchOrderServiceClient::new(
                    dispatch_order_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let throttle_workflow_channel = channel.clone();
    actions.insert(
        "throttle-workflow",
        Box::new(move |record| {
            ThrottleWorkflowSink {
                client: ratelimiting::throttle_workflow_service_client::ThrottleWorkflowServiceClient::new(
                    throttle_workflow_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
