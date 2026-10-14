//! `replay_warehouses_token` reads `acme.warehousing.v1.WarehouseService/SearchWarehouses` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = warehousing::warehouse_service_client::WarehouseServiceClient::new(channel.clone());
    let fetched = source
        .search_warehouses(warehousing::SearchWarehousesRequest {
            context: Some(context.clone()),
            order_by: "default-replay-warehouses-token".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = fetched.page_info.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = WarehousesTokenRecord {
        next_page_token: acme_kit::text::normalize(&page_info.next_page_token),
        status: result.status,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn WarehousesTokenSink>> = BTreeMap::new();
    sinks.insert(
        "access-grant-approval",
        Box::new(AccessGrantApprovalSink {
            client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "throttle-validation",
        Box::new(ThrottleValidationSink {
            client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "csr-workflow",
        Box::new(CsrWorkflowSink {
            client: certificates::csr_workflow_service_client::CsrWorkflowServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("access-grant-approval") else {
        anyhow::bail!("no sink registered for access-grant-approval");
    };
    sink.deliver(record).await
}

/// What `replay_warehouses_token` keeps of each SearchWarehouses response.
#[derive(Clone, Debug, Default)]
pub struct WarehousesTokenRecord {
    pub next_page_token: String,
    pub status: String,
    pub message: String,
}

/// Where `replay_warehouses_token` hands each record.
trait WarehousesTokenSink {
    fn deliver(&self, record: WarehousesTokenRecord) -> Delivery;
}

/// Writes records to `acme.secrets.v1.AccessGrantApprovalService`.
struct AccessGrantApprovalSink {
    client: secrets::access_grant_approval_service_client::AccessGrantApprovalServiceClient<Channel>,
}

impl WarehousesTokenSink for AccessGrantApprovalSink {
    fn deliver(&self, record: WarehousesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .submit_access_grant(secrets::SubmitAccessGrantRequest {
                    access_grant: Some(secrets::AccessGrant {
                        access_grant_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.ThrottleValidationService`.
struct ThrottleValidationSink {
    client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient<Channel>,
}

impl WarehousesTokenSink for ThrottleValidationSink {
    fn deliver(&self, record: WarehousesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_throttle(ratelimiting::ScoreThrottleRequest {
                    throttle_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CsrWorkflowService`.
struct CsrWorkflowSink {
    client: certificates::csr_workflow_service_client::CsrWorkflowServiceClient<Channel>,
}

impl WarehousesTokenSink for CsrWorkflowSink {
    fn deliver(&self, record: WarehousesTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pause_csr_workflow(certificates::PauseCsrWorkflowRequest {
                    csr_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
