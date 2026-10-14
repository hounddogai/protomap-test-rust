//! `export_get_api_keys_token` reads `acme.edgegateway.v1.ApiKeyService/BatchGetApiKeys` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::seating::v1 as seating;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = edgegateway::api_key_service_client::ApiKeyServiceClient::new(channel.clone());
    let fetched = source
        .batch_get_api_keys(edgegateway::BatchGetApiKeysRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let page_info = fetched.page_info.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = GetApiKeysTokenRecord {
        next_page_token: page_info.next_page_token,
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn GetApiKeysTokenWriter>> = BTreeMap::new();
    sinks.insert(
        "box-office-sale",
        Box::new(BoxOfficeSaleSink {
            client: boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "seat-map-calculation",
        Box::new(SeatMapCalculationSink {
            client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("box-office-sale") else {
        anyhow::bail!("no sink registered for box-office-sale");
    };
    sink.forward(record).await
}

/// What `export_get_api_keys_token` keeps of each BatchGetApiKeys response.
#[derive(Clone, Debug, Default)]
pub struct GetApiKeysTokenRecord {
    pub next_page_token: String,
    pub message: String,
    pub correlation_id: String,
}

/// Where `export_get_api_keys_token` hands each record.
trait GetApiKeysTokenWriter {
    fn forward(&self, record: GetApiKeysTokenRecord) -> Delivery;
}

/// Writes records to `acme.boxoffice.v1.BoxOfficeSaleService`.
struct BoxOfficeSaleSink {
    client: boxoffice::box_office_sale_service_client::BoxOfficeSaleServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetApiKeysTokenWriter for BoxOfficeSaleSink {
    fn forward(&self, record: GetApiKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(boxoffice::DeleteBoxOfficeSaleRequest {
                    etag: record.correlation_id,
                    box_office_sale_id: record.next_page_token,
                    reason: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.delete_box_office_sale(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.seating.v1.SeatMapCalculationService`.
struct SeatMapCalculationSink {
    client: seating::seat_map_calculation_service_client::SeatMapCalculationServiceClient<Channel>,
}

impl GetApiKeysTokenWriter for SeatMapCalculationSink {
    fn forward(&self, record: GetApiKeysTokenRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_seat_map_options(seating::CompareSeatMapOptionsRequest {
                    seat_map: Some(seating::SeatMap {
                        seat_map_id: record.next_page_token,
                        ..Default::default()
                    }),
                    reason: record.message,
                    details: Some(seating::compare_seat_map_options_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
