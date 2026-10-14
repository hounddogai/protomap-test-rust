//! `forward_certificate_authorities_message` reads
//! `acme.certificates.v1.CertificateAuthorityQueryService/CountCertificateAuthorities` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::inventory::v1 as inventory;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient::new(
            channel.clone(),
        );
    let reply = source
        .count_certificate_authorities(certificates::CountCertificateAuthoritiesRequest {
            context: Some(context.clone()),
            order_by: "default-forward-certificate-authorities-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = CertificateAuthoritiesMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        status: acme_kit::text::normalize(&result.status),
    };
    let sinks: Vec<(&str, Box<dyn CertificateAuthoritiesMessageTarget>)> = vec![
        ("flag-evaluation-notification", Box::new(FlagEvaluationNotificationSink { client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(channel.clone()), context: context.clone() })),
        ("stock-item-query", Box::new(StockItemQuerySink { client: inventory::stock_item_query_service_client::StockItemQueryServiceClient::new(channel.clone()) })),
        ("csr-lifecycle", Box::new(CsrLifecycleSink { client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient::new(channel) })),
    ];
    for (kind, sink) in &sinks {
        if *kind != "stock-item-query" {
            continue;
        }
        sink.deliver(record.clone()).await?;
    }
    Ok(())
}

/// What `forward_certificate_authorities_message` keeps of each CountCertificateAuthorities response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthoritiesMessageRecord {
    pub message: String,
    pub status: String,
}

/// Where `forward_certificate_authorities_message` hands each record.
trait CertificateAuthoritiesMessageTarget {
    fn deliver(&self, record: CertificateAuthoritiesMessageRecord) -> Delivery;
}

/// Passes records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificateAuthoritiesMessageTarget for FlagEvaluationNotificationSink {
    fn deliver(&self, record: CertificateAuthoritiesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::SendFlagEvaluationTestNoticeRequest {
                    details: Some(featureflags::send_flag_evaluation_test_notice_request::Details {
                        channel: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.send_flag_evaluation_test_notice(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.inventory.v1.StockItemQueryService`.
struct StockItemQuerySink {
    client: inventory::stock_item_query_service_client::StockItemQueryServiceClient<Channel>,
}

impl CertificateAuthoritiesMessageTarget for StockItemQuerySink {
    fn deliver(&self, record: CertificateAuthoritiesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_stock_items(inventory::SuggestStockItemsRequest {
                    order_by: record.status,
                    filter: Some(inventory::suggest_stock_items_request::Filter {
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CsrLifecycleService`.
struct CsrLifecycleSink {
    client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient<Channel>,
}

impl CertificateAuthoritiesMessageTarget for CsrLifecycleSink {
    fn deliver(&self, record: CertificateAuthoritiesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .terminate_csr(certificates::TerminateCsrRequest {
                    csr_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
