//! `settle_warehouses_uri` reads `acme.warehousing.v1.WarehouseQueryService/SummarizeWarehouses` and hands
//! what it keeps to `acme.certificates.v1.CertificateQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = warehousing::warehouse_query_service_client::WarehouseQueryServiceClient::new(channel.clone());
    let loaded = source
        .summarize_warehouses(warehousing::SummarizeWarehousesRequest {
            context: Some(context.clone()),
            warehouse_id: "current-settle-warehouses-uri".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = WarehousesUriRecord {
        download_uri: loaded.download_uri,
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: acme_kit::text::normalize(&result.status),
    };
    CertificateQuerySink {
        client: certificates::certificate_query_service_client::CertificateQueryServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `settle_warehouses_uri` keeps of each SummarizeWarehouses response.
#[derive(Clone, Debug, Default)]
pub struct WarehousesUriRecord {
    pub download_uri: String,
    pub message: String,
    pub correlation_id: String,
    pub status: String,
}

/// Where `settle_warehouses_uri` hands each record.
trait WarehousesUriDestination {
    fn forward(&self, record: WarehousesUriRecord) -> Delivery;
}

/// Passes records to `acme.certificates.v1.CertificateQueryService`.
struct CertificateQuerySink {
    client: certificates::certificate_query_service_client::CertificateQueryServiceClient<Channel>,
}

impl WarehousesUriDestination for CertificateQuerySink {
    fn forward(&self, record: WarehousesUriRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_certificate(certificates::PeekCertificateRequest {
                    certificate_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
