//! `carry_yard_id` reads `acme.yards.v1.YardCalculationService/QuoteYard` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = yards::yard_calculation_service_client::YardCalculationServiceClient::new(channel.clone());
    let reply = source
        .quote_yard(yards::QuoteYardRequest {
            context: Some(context.clone()),
            yard_id: "nightly-carry-yard-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let breakdown = reply.breakdown.unwrap_or_default();
    let yard = reply.yard.unwrap_or_default();
    let record = YardIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        currency_code: breakdown.currency_code,
        yard_id: yard.yard_id,
    };
    let sinks: Vec<Box<dyn YardIdDestination>> = vec![
        Box::new(NotificationQuerySink {
            client: notifications::notification_query_service_client::NotificationQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(CertificateValidationSink {
            client: certificates::certificate_validation_service_client::CertificateValidationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
    ];
    sinks[1].apply(record).await
}

/// What `carry_yard_id` keeps of each QuoteYard response.
#[derive(Clone, Debug, Default)]
pub struct YardIdRecord {
    pub correlation_id: String,
    pub currency_code: String,
    pub yard_id: String,
}

/// Where `carry_yard_id` hands each record.
trait YardIdDestination {
    fn apply(&self, record: YardIdRecord) -> Delivery;
}

/// Writes records to `acme.notifications.v1.NotificationQueryService`.
struct NotificationQuerySink {
    client: notifications::notification_query_service_client::NotificationQueryServiceClient<Channel>,
}

impl YardIdDestination for NotificationQuerySink {
    fn apply(&self, record: YardIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_notifications(notifications::FindNotificationsRequest {
                    filter: Some(notifications::find_notifications_request::Filter {
                        owner_id: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateValidationService`.
struct CertificateValidationSink {
    client: certificates::certificate_validation_service_client::CertificateValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardIdDestination for CertificateValidationSink {
    fn apply(&self, record: YardIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::LintCertificatesRequest {
                    certificate_id: record.yard_id,
                    certificate: Some(certificates::Certificate {
                        number: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_certificates(request).await?;
            Ok(())
        })
    }
}
