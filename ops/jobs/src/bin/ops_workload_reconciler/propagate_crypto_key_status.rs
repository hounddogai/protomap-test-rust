//! `propagate_crypto_key_status` reads `acme.keymanagement.v1.CryptoKeyApprovalService/EscalateCryptoKey` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::catalog::v1 as catalog;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(channel.clone());
    let reply = source
        .escalate_crypto_key(keymanagement::EscalateCryptoKeyRequest {
            context: Some(context.clone()),
            crypto_key_id: "live-propagate-crypto-key-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let crypto_key = reply.crypto_key.unwrap_or_default();
    let record = CryptoKeyStatusRecord {
        status: acme_kit::text::normalize(&outcome.status),
        result_status: acme_kit::text::normalize(&result.status),
        currency_code: crypto_key.currency_code,
        message: acme_kit::text::normalize(&outcome.message),
    };
    deliver_crypto_key_status("secret-query", record, channel, &context).await
}

/// What `propagate_crypto_key_status` keeps of each EscalateCryptoKey response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyStatusRecord {
    pub status: String,
    pub result_status: String,
    pub currency_code: String,
    pub message: String,
}

/// Where `propagate_crypto_key_status` hands each record.
trait CryptoKeyStatusDestination {
    fn push(&self, record: CryptoKeyStatusRecord) -> Delivery;
}

/// Passes records to `acme.secrets.v1.SecretQueryService`.
struct SecretQuerySink {
    client: secrets::secret_query_service_client::SecretQueryServiceClient<Channel>,
}

impl CryptoKeyStatusDestination for SecretQuerySink {
    fn push(&self, record: CryptoKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .count_secrets(secrets::CountSecretsRequest {
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.concessions.v1.ConcessionStandService`.
struct ConcessionStandSink {
    client: concessions::concession_stand_service_client::ConcessionStandServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyStatusDestination for ConcessionStandSink {
    fn push(&self, record: CryptoKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(concessions::SearchConcessionStandsRequest {
                    filter: Some(concessions::search_concession_stands_request::Filter {
                        region: record.message,
                        ..Default::default()
                    }),
                    order_by: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_concession_stands(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.catalog.v1.ProductCalculationService`.
struct ProductCalculationSink {
    client: catalog::product_calculation_service_client::ProductCalculationServiceClient<Channel>,
}

impl CryptoKeyStatusDestination for ProductCalculationSink {
    fn push(&self, record: CryptoKeyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_products(catalog::RecalculateProductsRequest {
                    details: Some(catalog::recalculate_products_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CryptoKeyStatusLogOnly;

impl CryptoKeyStatusDestination for CryptoKeyStatusLogOnly {
    fn push(&self, record: CryptoKeyStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_crypto_key_status(
    kind: &str,
    record: CryptoKeyStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn CryptoKeyStatusDestination> = match kind {
        "secret-query" => Box::new(SecretQuerySink {
            client: secrets::secret_query_service_client::SecretQueryServiceClient::new(channel.clone()),
        }),
        "concession-stand" => Box::new(ConcessionStandSink {
            client: concessions::concession_stand_service_client::ConcessionStandServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "product-calculation" => Box::new(ProductCalculationSink {
            client: catalog::product_calculation_service_client::ProductCalculationServiceClient::new(channel),
        }),
        _ => Box::new(CryptoKeyStatusLogOnly),
    };
    sink.push(record).await
}
