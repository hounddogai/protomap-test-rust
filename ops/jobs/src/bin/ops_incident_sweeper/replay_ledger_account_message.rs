//! `replay_ledger_account_message` reads `acme.ledger.v1.LedgerAccountService/CloneLedgerAccount` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::ledger::v1 as ledger;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::reviews::v1 as reviews;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ledger::ledger_account_service_client::LedgerAccountServiceClient::new(channel.clone());
    let found = source
        .clone_ledger_account(ledger::CloneLedgerAccountRequest {
            context: Some(context.clone()),
            request_id: "live-replay-ledger-account-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let ledger_account = found.ledger_account.unwrap_or_default();
    let record = LedgerAccountMessageRecord {
        message: result.message,
        currency_code: acme_kit::text::normalize(&ledger_account.currency_code),
        correlation_id: result.correlation_id,
    };
    act_on_ledger_account_message("throttle-calculation", record, channel, context).await
}

/// What `replay_ledger_account_message` keeps of each CloneLedgerAccount response.
#[derive(Clone, Debug, Default)]
pub struct LedgerAccountMessageRecord {
    pub message: String,
    pub currency_code: String,
    pub correlation_id: String,
}

/// Where `replay_ledger_account_message` hands each record.
trait LedgerAccountMessageTarget {
    fn send(&self, record: LedgerAccountMessageRecord) -> Delivery;
}

/// Writes records to `acme.reviews.v1.ProductReviewService`.
struct ProductReviewSink {
    client: reviews::product_review_service_client::ProductReviewServiceClient<Channel>,
}

impl LedgerAccountMessageTarget for ProductReviewSink {
    fn send(&self, record: LedgerAccountMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_product_review(reviews::CloneProductReviewRequest {
                    request_id: record.currency_code,
                    product_review: Some(reviews::ProductReview {
                        product_review_id: record.correlation_id,
                        number: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.ratelimiting.v1.ThrottleCalculationService`.
struct ThrottleCalculationSink {
    client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl LedgerAccountMessageTarget for ThrottleCalculationSink {
    fn send(&self, record: LedgerAccountMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::CalculateThrottleRequest {
                    details: Some(ratelimiting::calculate_throttle_request::Details {
                        currency_code: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.calculate_throttle(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_ledger_account_message(
    kind: &str,
    record: LedgerAccountMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(LedgerAccountMessageRecord) -> Delivery>> = BTreeMap::new();
    let product_review_channel = channel.clone();
    actions.insert(
        "product-review",
        Box::new(move |record| {
            ProductReviewSink {
                client: reviews::product_review_service_client::ProductReviewServiceClient::new(
                    product_review_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    let throttle_calculation_channel = channel.clone();
    let throttle_calculation_context = context.clone();
    actions.insert(
        "throttle-calculation",
        Box::new(move |record| {
            ThrottleCalculationSink {
                client: ratelimiting::throttle_calculation_service_client::ThrottleCalculationServiceClient::new(
                    throttle_calculation_channel.clone(),
                ),
                context: throttle_calculation_context.clone(),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
