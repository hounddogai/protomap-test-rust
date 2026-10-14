//! `hand_off_secret_id` reads `acme.secrets.v1.SecretService/DeleteSecret` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = secrets::secret_service_client::SecretServiceClient::new(channel.clone());
    let loaded = source
        .delete_secret(secrets::DeleteSecretRequest {
            context: Some(context.clone()),
            etag: "nightly-hand-off-secret-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = OffSecretIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffSecretIdTarget>> = BTreeMap::new();
    sinks.insert(
        "key-ring-approval",
        Box::new(KeyRingApprovalSink {
            client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "seller-query",
        Box::new(SellerQuerySink {
            client: sellers::seller_query_service_client::SellerQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "document-query",
        Box::new(DocumentQuerySink {
            client: documents::document_query_service_client::DocumentQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "gift-card-calculation",
        Box::new(GiftCardCalculationSink {
            client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient::new(channel),
        }),
    );
    match sinks.get("gift-card-calculation") {
        Some(sink) => sink.forward(record).await,
        None => anyhow::bail!("no sink registered for gift-card-calculation"),
    }
}

/// What `hand_off_secret_id` keeps of each DeleteSecret response.
#[derive(Clone, Debug, Default)]
pub struct OffSecretIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub message: String,
}

/// Where `hand_off_secret_id` hands each record.
trait OffSecretIdTarget {
    fn forward(&self, record: OffSecretIdRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRingApprovalService`.
struct KeyRingApprovalSink {
    client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient<Channel>,
}

impl OffSecretIdTarget for KeyRingApprovalSink {
    fn forward(&self, record: OffSecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_key_ring_approval_status(keymanagement::GetKeyRingApprovalStatusRequest {
                    key_ring_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.sellers.v1.SellerQueryService`.
struct SellerQuerySink {
    client: sellers::seller_query_service_client::SellerQueryServiceClient<Channel>,
}

impl OffSecretIdTarget for SellerQuerySink {
    fn forward(&self, record: OffSecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_seller(sellers::LookupSellerRequest {
                    seller_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.documents.v1.DocumentQueryService`.
struct DocumentQuerySink {
    client: documents::document_query_service_client::DocumentQueryServiceClient<Channel>,
}

impl OffSecretIdTarget for DocumentQuerySink {
    fn forward(&self, record: OffSecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_documents(documents::BrowseDocumentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    filter: Some(documents::browse_documents_request::Filter {
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.giftcards.v1.GiftCardCalculationService`.
struct GiftCardCalculationSink {
    client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient<Channel>,
}

impl OffSecretIdTarget for GiftCardCalculationSink {
    fn forward(&self, record: OffSecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_gift_card(giftcards::CalculateGiftCardRequest {
                    reason: record.status,
                    gift_card: Some(giftcards::GiftCard {
                        gift_card_id: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(giftcards::calculate_gift_card_request::Details {
                        currency_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
