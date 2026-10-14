//! `propagate_box_office_sales_status` reads
//! `acme.boxoffice.v1.BoxOfficeSaleQueryService/ResolveBoxOfficeSales` and hands what it keeps to the sink of
//! its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::boxoffice::v1 as boxoffice;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sponsorships::v1 as sponsorships;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        boxoffice::box_office_sale_query_service_client::BoxOfficeSaleQueryServiceClient::new(channel.clone());
    let reply = source
        .resolve_box_office_sales(boxoffice::ResolveBoxOfficeSalesRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = BoxOfficeSalesStatusRecord {
        status: result.status,
        next_page_token: page_info.next_page_token,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    let mut sinks: BTreeMap<&str, Box<dyn BoxOfficeSalesStatusWriter>> = BTreeMap::new();
    sinks.insert(
        "yard",
        Box::new(YardSink {
            client: yards::yard_service_client::YardServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "crypto-key-approval",
        Box::new(CryptoKeyApprovalSink {
            client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "sponsor-query",
        Box::new(SponsorQuerySink {
            client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient::new(channel),
        }),
    );
    match sinks.get("yard") {
        Some(sink) => sink.forward(record).await,
        None => anyhow::bail!("no sink registered for yard"),
    }
}

/// What `propagate_box_office_sales_status` keeps of each ResolveBoxOfficeSales response.
#[derive(Clone, Debug, Default)]
pub struct BoxOfficeSalesStatusRecord {
    pub status: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `propagate_box_office_sales_status` hands each record.
trait BoxOfficeSalesStatusWriter {
    fn forward(&self, record: BoxOfficeSalesStatusRecord) -> Delivery;
}

/// Hands records to `acme.yards.v1.YardService`.
struct YardSink {
    client: yards::yard_service_client::YardServiceClient<Channel>,
}

impl BoxOfficeSalesStatusWriter for YardSink {
    fn forward(&self, record: BoxOfficeSalesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_yard(yards::CloneYardRequest {
                    request_id: record.correlation_id,
                    yard: Some(yards::Yard {
                        name: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.keymanagement.v1.CryptoKeyApprovalService`.
struct CryptoKeyApprovalSink {
    client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient<Channel>,
}

impl BoxOfficeSalesStatusWriter for CryptoKeyApprovalSink {
    fn forward(&self, record: BoxOfficeSalesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .submit_crypto_key(keymanagement::SubmitCryptoKeyRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        crypto_key_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.sponsorships.v1.SponsorQueryService`.
struct SponsorQuerySink {
    client: sponsorships::sponsor_query_service_client::SponsorQueryServiceClient<Channel>,
}

impl BoxOfficeSalesStatusWriter for SponsorQuerySink {
    fn forward(&self, record: BoxOfficeSalesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_sponsors(sponsorships::SuggestSponsorsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
