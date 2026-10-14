//! `mirror_warehouse_status` reads `acme.warehousing.v1.WarehouseService/GetWarehouse` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = warehousing::warehouse_service_client::WarehouseServiceClient::new(channel.clone());
    let found = source
        .get_warehouse(warehousing::GetWarehouseRequest {
            context: Some(context.clone()),
            warehouse_id: "nightly-mirror-warehouse-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let warehouse = found.warehouse.unwrap_or_default();
    let record = WarehouseStatusRecord {
        status: result.status,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        code: warehouse.code,
        name: warehouse.name,
    };
    deliver_warehouse_status("broadcast-query", record, channel, &context).await
}

/// What `mirror_warehouse_status` keeps of each GetWarehouse response.
#[derive(Clone, Debug, Default)]
pub struct WarehouseStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub code: String,
    pub name: String,
}

/// Where `mirror_warehouse_status` hands each record.
trait WarehouseStatusSink {
    fn apply(&self, record: WarehouseStatusRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.RoleBindingAdminService`.
struct RoleBindingAdminSink {
    client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl WarehouseStatusSink for RoleBindingAdminSink {
    fn apply(&self, record: WarehouseStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::PurgeRoleBindingRequest {
                    etag: record.status,
                    reason: record.name,
                    role_binding_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.purge_role_binding(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CsrApprovalService`.
struct CsrApprovalSink {
    client: certificates::csr_approval_service_client::CsrApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl WarehouseStatusSink for CsrApprovalSink {
    fn apply(&self, record: WarehouseStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::GetCsrApprovalStatusRequest {
                    csr_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_csr_approval_status(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.broadcasts.v1.BroadcastQueryService`.
struct BroadcastQuerySink {
    client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl WarehouseStatusSink for BroadcastQuerySink {
    fn apply(&self, record: WarehouseStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(broadcasts::CountBroadcastsRequest {
                    filter: Some(broadcasts::count_broadcasts_request::Filter {
                        region: record.name,
                        query: record.correlation_id,
                        owner_id: record.code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_broadcasts(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct WarehouseStatusLogOnly;

impl WarehouseStatusSink for WarehouseStatusLogOnly {
    fn apply(&self, record: WarehouseStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_warehouse_status(
    kind: &str,
    record: WarehouseStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn WarehouseStatusSink> = match kind {
        "role-binding-admin" => Box::new(RoleBindingAdminSink {
            client: authorization::role_binding_admin_service_client::RoleBindingAdminServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "csr-approval" => Box::new(CsrApprovalSink {
            client: certificates::csr_approval_service_client::CsrApprovalServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "broadcast-query" => Box::new(BroadcastQuerySink {
            client: broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(WarehouseStatusLogOnly),
    };
    sink.apply(record).await
}
