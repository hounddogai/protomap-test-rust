//! `hand_off_deliveries_id` reads `acme.deliveries.v1.DeliveryQueryService/FindDeliveries` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone());
    let fetched = source
        .find_deliveries(deliveries::FindDeliveriesRequest {
            context: Some(context.clone()),
            order_by: "nightly-hand-off-deliveries-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = OffDeliveriesIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
        status: result.status,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffDeliveriesIdSink>> = BTreeMap::new();
    sinks.insert(
        "crypto-key-lifecycle",
        Box::new(CryptoKeyLifecycleSink {
            client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "fan-session",
        Box::new(FanSessionSink {
            client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert("certificate-authority-calculation", Box::new(CertificateAuthorityCalculationSink { client: certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient::new(channel) }));
    let Some(sink) = sinks.get("fan-session") else {
        anyhow::bail!("no sink registered for fan-session");
    };
    sink.push(record).await
}

/// What `hand_off_deliveries_id` keeps of each FindDeliveries response.
#[derive(Clone, Debug, Default)]
pub struct OffDeliveriesIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub status: String,
    pub message: String,
}

/// Where `hand_off_deliveries_id` hands each record.
trait OffDeliveriesIdSink {
    fn push(&self, record: OffDeliveriesIdRecord) -> Delivery;
}

/// Writes records to `acme.keymanagement.v1.CryptoKeyLifecycleService`.
struct CryptoKeyLifecycleSink {
    client: keymanagement::crypto_key_lifecycle_service_client::CryptoKeyLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffDeliveriesIdSink for CryptoKeyLifecycleSink {
    fn push(&self, record: OffDeliveriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::SuspendCryptoKeyRequest {
                    reason: record.next_page_token,
                    details: Some(keymanagement::suspend_crypto_key_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suspend_crypto_key(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
    context: common::RequestContext,
}

impl OffDeliveriesIdSink for FanSessionSink {
    fn push(&self, record: OffDeliveriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fanportal::SearchFanSessionsRequest {
                    filter: Some(fanportal::search_fan_sessions_request::Filter {
                        owner_id: record.next_page_token,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    page: Some(common::PageRequest {
                        page_token: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_fan_sessions(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateAuthorityCalculationService`.
struct CertificateAuthorityCalculationSink {
    client:
        certificates::certificate_authority_calculation_service_client::CertificateAuthorityCalculationServiceClient<
            Channel,
        >,
}

impl OffDeliveriesIdSink for CertificateAuthorityCalculationSink {
    fn push(&self, record: OffDeliveriesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_certificate_authority_rates(certificates::GetCertificateAuthorityRatesRequest {
                    certificate_authority_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
