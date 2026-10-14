//! `propagate_warehouse_name` reads `acme.warehousing.v1.WarehouseService/UpdateWarehouse` and hands what it
//! keeps to `acme.concessions.v1.ConcessionStandService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::warehousing::v1 as warehousing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = warehousing::warehouse_service_client::WarehouseServiceClient::new(channel.clone());
    let reply = source
        .update_warehouse(warehousing::UpdateWarehouseRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let warehouse = reply.warehouse.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = WarehouseNameRecord {
        name: acme_kit::text::normalize(&warehouse.name),
        status: result.status,
    };
    ConcessionStandSink {
        client: concessions::concession_stand_service_client::ConcessionStandServiceClient::new(channel),
    }
    .apply(record)
    .await
}

/// What `propagate_warehouse_name` keeps of each UpdateWarehouse response.
#[derive(Clone, Debug, Default)]
pub struct WarehouseNameRecord {
    pub name: String,
    pub status: String,
}

/// Where `propagate_warehouse_name` hands each record.
trait WarehouseNameSink {
    fn apply(&self, record: WarehouseNameRecord) -> Delivery;
}

/// Passes records to `acme.concessions.v1.ConcessionStandService`.
struct ConcessionStandSink {
    client: concessions::concession_stand_service_client::ConcessionStandServiceClient<Channel>,
}

impl WarehouseNameSink for ConcessionStandSink {
    fn apply(&self, record: WarehouseNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_concession_stand(concessions::UpdateConcessionStandRequest {
                    concession_stand: Some(concessions::ConcessionStand {
                        code: record.status,
                        concession_stand_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
