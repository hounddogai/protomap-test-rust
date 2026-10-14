//! `backfill_tours_id` reads `acme.tours.v1.TourQueryService/BrowseTours` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = tours::tour_query_service_client::TourQueryServiceClient::new(channel.clone());
    let found = source
        .browse_tours(tours::BrowseToursRequest {
            context: Some(context.clone()),
            order_by: "current-backfill-tours-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = ToursIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: result.status,
        next_page_token: page_info.next_page_token,
        message: result.message,
    };
    let sinks: Vec<(&str, Box<dyn ToursIdDestination>)> = vec![
        (
            "box-office-sale-query",
            Box::new(BoxOfficeSaleQuerySink {
                client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "key-rotation-validation",
            Box::new(KeyRotationValidationSink {
                client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient::new(
                    channel,
                ),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "key-rotation-validation" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `backfill_tours_id` keeps of each BrowseTours response.
#[derive(Clone, Debug, Default)]
pub struct ToursIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
    pub message: String,
}

/// Where `backfill_tours_id` hands each record.
trait ToursIdDestination {
    fn deliver(&self, record: ToursIdRecord) -> Delivery;
}

/// Writes records to `acme.boxoffice.v1.BoxOfficeSaleQueryService`.
struct BoxOfficeSaleQuerySink {
    client: boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient<Channel>,
}

impl ToursIdDestination for BoxOfficeSaleQuerySink {
    fn deliver(&self, record: ToursIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_box_office_sales(boxoffice::BrowseBoxOfficeSalesRequest {
                    filter: Some(boxoffice::browse_box_office_sales_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    order_by: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRotationValidationService`.
struct KeyRotationValidationSink {
    client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ToursIdDestination for KeyRotationValidationSink {
    fn deliver(&self, record: ToursIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::PreviewKeyRotationRequest {
                    reason: record.next_page_token,
                    key_rotation: Some(keymanagement::KeyRotation {
                        title: record.status,
                        key_rotation_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.preview_key_rotation(request).await?;
            Ok(())
        })
    }
}
