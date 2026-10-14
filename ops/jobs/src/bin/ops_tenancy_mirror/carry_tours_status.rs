//! `carry_tours_status` reads `acme.tours.v1.TourService/SearchTours` and hands what it keeps to the sink of
//! its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::reviews::v1 as reviews;
use acme_proto::acme::tours::v1 as tours;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = tours::tour_service_client::TourServiceClient::new(channel.clone());
    let found = source
        .search_tours(tours::SearchToursRequest {
            context: Some(context.clone()),
            order_by: "current-carry-tours-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = ToursStatusRecord {
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    act_on_tours_status("crypto-key-validation", record, channel, context).await
}

/// What `carry_tours_status` keeps of each SearchTours response.
#[derive(Clone, Debug, Default)]
pub struct ToursStatusRecord {
    pub status: String,
    pub next_page_token: String,
}

/// Where `carry_tours_status` hands each record.
trait ToursStatusSink {
    fn write(&self, record: ToursStatusRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.CryptoKeyValidationService`.
struct CryptoKeyValidationSink {
    client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ToursStatusSink for CryptoKeyValidationSink {
    fn write(&self, record: ToursStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::ValidateCryptoKeyRequest {
                    details: Some(keymanagement::validate_crypto_key_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    crypto_key_id: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.validate_crypto_key(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
}

impl ToursStatusSink for ThrottleCalculationSink {
    fn write(&self, record: ToursStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_throttles(ratelimiting::RecalculateThrottlesRequest {
                    throttle: Some(ratelimiting::Throttle {
                        source: record.status,
                        ..Default::default()
                    }),
                    details: Some(ratelimiting::recalculate_throttles_request::Details {
                        currency_code: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.reviews.v1.ProductReviewService`.
struct ProductReviewSink {
    client: reviews::product_review_service_client::ProductReviewServiceClient<Channel>,
}

impl ToursStatusSink for ProductReviewSink {
    fn write(&self, record: ToursStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_product_review(reviews::DeleteProductReviewRequest {
                    product_review_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_tours_status(
    kind: &str,
    record: ToursStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ToursStatusRecord) -> Delivery>> = BTreeMap::new();
    let crypto_key_validation_channel = channel.clone();
    let crypto_key_validation_context = context.clone();
    actions.insert(
        "crypto-key-validation",
        Box::new(move |record| {
            CryptoKeyValidationSink {
                client: keymanagement::crypto_key_validation_service_client::CryptoKeyValidationServiceClient::new(
                    crypto_key_validation_channel.clone(),
                ),
                context: crypto_key_validation_context.clone(),
            }
            .write(record)
        }),
    );
    let throttle_calculation_channel = channel.clone();
    actions.insert(
        "throttle-calculation",
        Box::new(move |record| {
            ThrottleCalculationSink {
                client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(
                    throttle_calculation_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let product_review_channel = channel.clone();
    actions.insert(
        "product-review",
        Box::new(move |record| {
            ProductReviewSink {
                client: reviews::product_review_service_client::ProductReviewServiceClient::new(
                    product_review_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
