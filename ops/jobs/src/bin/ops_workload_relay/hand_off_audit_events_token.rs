//! `hand_off_audit_events_token` reads `acme.audit.v1.AuditEventQueryService/BrowseAuditEvents` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = audit::audit_event_query_service_client::AuditEventQueryServiceClient::new(channel.clone());
    let fetched = source
        .browse_audit_events(audit::BrowseAuditEventsRequest {
            context: Some(context.clone()),
            order_by: "main-hand-off-audit-events-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = fetched.page_info.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = OffAuditEventsTokenRecord {
        next_page_token: page_info.next_page_token,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffAuditEventsTokenTarget>> = BTreeMap::new();
    sinks.insert(
        "permission-check-admin",
        Box::new(PermissionCheckAdminSink {
            client: authorization::permission_check_admin_service_client::PermissionCheckAdminServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "warehouse-query",
        Box::new(WarehouseQuerySink {
            client: warehousing::warehouse_query_service_client::WarehouseQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("warehouse-query") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for warehouse-query"),
    }
}

/// What `hand_off_audit_events_token` keeps of each BrowseAuditEvents response.
#[derive(Clone, Debug, Default)]
pub struct OffAuditEventsTokenRecord {
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `hand_off_audit_events_token` hands each record.
trait OffAuditEventsTokenTarget {
    fn write(&self, record: OffAuditEventsTokenRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.PermissionCheckAdminService`.
struct PermissionCheckAdminSink {
    client: authorization::permission_check_admin_service_client::PermissionCheckAdminServiceClient<Channel>,
}

impl OffAuditEventsTokenTarget for PermissionCheckAdminSink {
    fn write(&self, record: OffAuditEventsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lock_permission_check(authorization::LockPermissionCheckRequest {
                    permission_check_id: record.correlation_id,
                    reason: record.next_page_token,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.warehousing.v1.WarehouseQueryService`.
struct WarehouseQuerySink {
    client: warehousing::warehouse_query_service_client::WarehouseQueryServiceClient<Channel>,
}

impl OffAuditEventsTokenTarget for WarehouseQuerySink {
    fn write(&self, record: OffAuditEventsTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_warehouses(warehousing::SuggestWarehousesRequest {
                    filter: Some(warehousing::suggest_warehouses_request::Filter {
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
