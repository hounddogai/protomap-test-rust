//! `propagate_checkout_sessions_timezone` reads
//! `acme.checkout.v1.CheckoutSessionQueryService/SummarizeCheckoutSessions` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::checkout::v1 as checkout;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::notifications::v1 as notifications;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        checkout::checkout_session_query_service_client::CheckoutSessionQueryServiceClient::new(channel.clone());
    let fetched = source
        .summarize_checkout_sessions(checkout::SummarizeCheckoutSessionsRequest {
            context: Some(context.clone()),
            reason: "primary-propagate-checkout-sessions-timezone".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let period = fetched.period.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = CheckoutSessionsTimezoneRecord {
        timezone: period.timezone,
        download_uri: fetched.download_uri,
        status: acme_kit::text::normalize(&result.status),
        report_id: fetched.report_id,
    };
    dispatch_checkout_sessions_timezone("certificate-calculation", record, channel, &context).await
}

/// What `propagate_checkout_sessions_timezone` keeps of each SummarizeCheckoutSessions response.
#[derive(Clone, Debug, Default)]
pub struct CheckoutSessionsTimezoneRecord {
    pub timezone: String,
    pub download_uri: String,
    pub status: String,
    pub report_id: String,
}

/// Where `propagate_checkout_sessions_timezone` hands each record.
trait CheckoutSessionsTimezoneDestination {
    fn send(&self, record: CheckoutSessionsTimezoneRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CertificateCalculationService`.
struct CertificateCalculationSink {
    client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient<Channel>,
}

impl CheckoutSessionsTimezoneDestination for CertificateCalculationSink {
    fn send(&self, record: CheckoutSessionsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_certificates(certificates::RecalculateCertificatesRequest {
                    certificate_id: record.report_id,
                    certificate: Some(certificates::Certificate {
                        number: record.timezone,
                        notes: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.notifications.v1.NotificationService`.
struct NotificationSink {
    client: notifications::notification_service_client::NotificationServiceClient<Channel>,
    context: common::RequestContext,
}

impl CheckoutSessionsTimezoneDestination for NotificationSink {
    fn send(&self, record: CheckoutSessionsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(notifications::GetNotificationRequest {
                    notification_id: record.report_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_notification(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.featureflags.v1.FeatureFlagHistoryService`.
struct FeatureFlagHistorySink {
    client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient<Channel>,
}

impl CheckoutSessionsTimezoneDestination for FeatureFlagHistorySink {
    fn send(&self, record: CheckoutSessionsTimezoneRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_feature_flag_version(featureflags::GetFeatureFlagVersionRequest {
                    feature_flag_id: record.timezone,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CheckoutSessionsTimezoneLogOnly;

impl CheckoutSessionsTimezoneDestination for CheckoutSessionsTimezoneLogOnly {
    fn send(&self, record: CheckoutSessionsTimezoneRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.timezone);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_checkout_sessions_timezone(
    kind: &str,
    record: CheckoutSessionsTimezoneRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let certificate_calculation = CertificateCalculationSink {
        client: certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let notification = NotificationSink {
        client: notifications::notification_service_client::NotificationServiceClient::new(channel.clone()),
        context: context.clone(),
    };
    let feature_flag_history = FeatureFlagHistorySink {
        client: featureflags::feature_flag_history_service_client::FeatureFlagHistoryServiceClient::new(channel),
    };
    let write: &dyn Fn(CheckoutSessionsTimezoneRecord) -> Delivery = match kind {
        "certificate-calculation" => &|record| certificate_calculation.send(record),
        "notification" => &|record| notification.send(record),
        "feature-flag-history" => &|record| feature_flag_history.send(record),
        _ => &|record| CheckoutSessionsTimezoneLogOnly.send(record),
    };
    write(record).await
}
