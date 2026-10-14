//! `route_get_contracts_status` reads `acme.contracts.v1.ContractService/BatchGetContracts` and hands what it
//! keeps to `acme.routing.v1.RouteQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::contracts::v1 as contracts;
use acme_proto::acme::routing::v1 as routing;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = contracts::contract_service_client::ContractServiceClient::new(channel.clone());
    let loaded = source
        .batch_get_contracts(contracts::BatchGetContractsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = GetContractsStatusRecord {
        status: result.status,
        message: result.message,
        next_page_token: page_info.next_page_token,
    };
    RouteQuerySink {
        client: routing::route_query_service_client::RouteQueryServiceClient::new(channel),
    }
    .push(record)
    .await
}

/// What `route_get_contracts_status` keeps of each BatchGetContracts response.
#[derive(Clone, Debug, Default)]
pub struct GetContractsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
}

/// Where `route_get_contracts_status` hands each record.
trait GetContractsStatusSink {
    fn push(&self, record: GetContractsStatusRecord) -> Delivery;
}

/// Passes records to `acme.routing.v1.RouteQueryService`.
struct RouteQuerySink {
    client: routing::route_query_service_client::RouteQueryServiceClient<Channel>,
}

impl GetContractsStatusSink for RouteQuerySink {
    fn push(&self, record: GetContractsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_route(routing::LookupRouteRequest {
                    route_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
