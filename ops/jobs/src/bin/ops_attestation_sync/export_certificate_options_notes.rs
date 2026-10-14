//! `export_certificate_options_notes` reads
//! `acme.certificates.v1.CertificateCalculationService/CompareCertificateOptions` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::giftcards::v1 as giftcards;
use acme_proto::acme::notifications::v1 as notifications;
use acme_proto::acme::telehealth::v1 as telehealth;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_calculation_service_client::CertificateCalculationServiceClient::new(channel.clone());
    let loaded = source
        .compare_certificate_options(certificates::CompareCertificateOptionsRequest {
            context: Some(context.clone()),
            reason: "current-export-certificate-options-notes".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate = loaded.certificate.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let record = CertificateOptionsNotesRecord {
        notes: certificate.notes,
        message: result.message,
        currency_code: breakdown.currency_code,
    };
    act_on_certificate_options_notes("document-validation", record, channel, context).await
}

/// What `export_certificate_options_notes` keeps of each CompareCertificateOptions response.
#[derive(Clone, Debug, Default)]
pub struct CertificateOptionsNotesRecord {
    pub notes: String,
    pub message: String,
    pub currency_code: String,
}

/// Where `export_certificate_options_notes` hands each record.
trait CertificateOptionsNotesWriter {
    fn apply(&self, record: CertificateOptionsNotesRecord) -> Delivery;
}

/// Sends records to `acme.documents.v1.DocumentValidationService`.
struct DocumentValidationSink {
    client: documents::document_validation_service_client::DocumentValidationServiceClient<Channel>,
}

impl CertificateOptionsNotesWriter for DocumentValidationSink {
    fn apply(&self, record: CertificateOptionsNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .validate_document(documents::ValidateDocumentRequest {
                    details: Some(documents::validate_document_request::Details {
                        locale: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.notifications.v1.NotificationQueryService`.
struct NotificationQuerySink {
    client: notifications::notification_query_service_client::NotificationQueryServiceClient<Channel>,
}

impl CertificateOptionsNotesWriter for NotificationQuerySink {
    fn apply(&self, record: CertificateOptionsNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_notification(notifications::PeekNotificationRequest {
                    notification_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.giftcards.v1.GiftCardCalculationService`.
struct GiftCardCalculationSink {
    client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient<Channel>,
}

impl CertificateOptionsNotesWriter for GiftCardCalculationSink {
    fn apply(&self, record: CertificateOptionsNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_gift_card(giftcards::SimulateGiftCardRequest {
                    details: Some(giftcards::simulate_gift_card_request::Details {
                        currency_code: record.currency_code,
                        ..Default::default()
                    }),
                    gift_card_id: record.message,
                    reason: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.telehealth.v1.VideoVisitCalculationService`.
struct VideoVisitCalculationSink {
    client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient<Channel>,
}

impl CertificateOptionsNotesWriter for VideoVisitCalculationSink {
    fn apply(&self, record: CertificateOptionsNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_video_visit(telehealth::SimulateVideoVisitRequest {
                    video_visit: Some(telehealth::VideoVisit {
                        title: record.message,
                        ..Default::default()
                    }),
                    video_visit_id: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_certificate_options_notes(
    kind: &str,
    record: CertificateOptionsNotesRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CertificateOptionsNotesRecord) -> Delivery>> = BTreeMap::new();
    let document_validation_channel = channel.clone();
    actions.insert(
        "document-validation",
        Box::new(move |record| {
            DocumentValidationSink {
                client: documents::document_validation_service_client::DocumentValidationServiceClient::new(
                    document_validation_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let notification_query_channel = channel.clone();
    actions.insert(
        "notification-query",
        Box::new(move |record| {
            NotificationQuerySink {
                client: notifications::notification_query_service_client::NotificationQueryServiceClient::new(
                    notification_query_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let gift_card_calculation_channel = channel.clone();
    actions.insert(
        "gift-card-calculation",
        Box::new(move |record| {
            GiftCardCalculationSink {
                client: giftcards::gift_card_calculation_service_client::GiftCardCalculationServiceClient::new(
                    gift_card_calculation_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    let video_visit_calculation_channel = channel.clone();
    actions.insert(
        "video-visit-calculation",
        Box::new(move |record| {
            VideoVisitCalculationSink {
                client: telehealth::video_visit_calculation_service_client::VideoVisitCalculationServiceClient::new(
                    video_visit_calculation_channel.clone(),
                ),
            }
            .apply(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
