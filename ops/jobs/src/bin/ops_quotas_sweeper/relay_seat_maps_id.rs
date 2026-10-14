//! `relay_seat_maps_id` reads `acme.seating.v1.SeatMapQueryService/CountSeatMaps` and hands what it keeps to
//! `acme.customers.v1.CustomerQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::customers::v1 as customers;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = seating::seat_map_query_service_client::SeatMapQueryServiceClient::new(channel.clone());
    let found = source
        .count_seat_maps(seating::CountSeatMapsRequest {
            context: Some(context.clone()),
            order_by: "current-relay-seat-maps-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = SeatMapsIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        message: result.message,
    };
    CustomerQuerySink {
        client: customers::customer_query_service_client::CustomerQueryServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `relay_seat_maps_id` keeps of each CountSeatMaps response.
#[derive(Clone, Debug, Default)]
pub struct SeatMapsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub message: String,
}

/// Where `relay_seat_maps_id` hands each record.
trait SeatMapsIdSink {
    fn forward(&self, record: SeatMapsIdRecord) -> Delivery;
}

/// Sends records to `acme.customers.v1.CustomerQueryService`.
struct CustomerQuerySink {
    client: customers::customer_query_service_client::CustomerQueryServiceClient<Channel>,
}

impl SeatMapsIdSink for CustomerQuerySink {
    fn forward(&self, record: SeatMapsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_customers(customers::CountCustomersRequest {
                    filter: Some(customers::count_customers_request::Filter {
                        owner_id: record.correlation_id,
                        region: record.status,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
