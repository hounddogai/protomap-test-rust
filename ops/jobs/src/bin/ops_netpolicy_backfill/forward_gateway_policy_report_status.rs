//! `forward_gateway_policy_report_status` reads
//! `acme.edgegateway.v1.GatewayPolicyReportingService/DeleteGatewayPolicyReport` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient::new(channel.clone());
    let fetched = source
        .delete_gateway_policy_report(edgegateway::DeleteGatewayPolicyReportRequest {
            context: Some(context.clone()),
            etag: "nightly-forward-gateway-policy-report-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = GatewayPolicyReportStatusRecord {
        status: result.status,
        message: result.message,
        correlation_id: result.correlation_id,
    };
    deliver_gateway_policy_report_status("key-rotation-lifecycle", record, channel, &context).await
}

/// What `forward_gateway_policy_report_status` keeps of each DeleteGatewayPolicyReport response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyReportStatusRecord {
    pub status: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `forward_gateway_policy_report_status` hands each record.
trait GatewayPolicyReportStatusTarget {
    fn write(&self, record: GatewayPolicyReportStatusRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.RoleBindingHistoryService`.
struct RoleBindingHistorySink {
    client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient<Channel>,
    context: common::RequestContext,
}

impl GatewayPolicyReportStatusTarget for RoleBindingHistorySink {
    fn write(&self, record: GatewayPolicyReportStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::PruneRoleBindingVersionsRequest {
                    role_binding_id: record.message,
                    etag: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.prune_role_binding_versions(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.KeyRotationLifecycleService`.
struct KeyRotationLifecycleSink {
    client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl GatewayPolicyReportStatusTarget for KeyRotationLifecycleSink {
    fn write(&self, record: GatewayPolicyReportStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ExpireKeyRotationRequest {
                    reason: record.correlation_id,
                    details: Some(keymanagement::expire_key_rotation_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.expire_key_rotation(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct GatewayPolicyReportStatusLogOnly;

impl GatewayPolicyReportStatusTarget for GatewayPolicyReportStatusLogOnly {
    fn write(&self, record: GatewayPolicyReportStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_gateway_policy_report_status(
    kind: &str,
    record: GatewayPolicyReportStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn GatewayPolicyReportStatusTarget> = match kind {
        "role-binding-history" => Box::new(RoleBindingHistorySink {
            client: authorization::role_binding_history_service_client::RoleBindingHistoryServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "key-rotation-lifecycle" => Box::new(KeyRotationLifecycleSink {
            client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
        _ => Box::new(GatewayPolicyReportStatusLogOnly),
    };
    sink.write(record).await
}
