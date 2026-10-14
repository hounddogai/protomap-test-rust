//! `mirror_certificate_authority_message` reads
//! `acme.certificates.v1.CertificateAuthorityApprovalService/EscalateCertificateAuthority` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::helpdesk::v1 as helpdesk;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        certificates::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient::new(
            channel.clone(),
        );
    let found = source
        .escalate_certificate_authority(certificates::EscalateCertificateAuthorityRequest {
            context: Some(context.clone()),
            certificate_authority_id: "nightly-mirror-certificate-authority-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = found.outcome.unwrap_or_default();
    let certificate_authority = found.certificate_authority.unwrap_or_default();
    let record = CertificateAuthorityMessageRecord {
        message: outcome.message,
        parent_id: certificate_authority.parent_id,
        certificate_authority_id: certificate_authority.certificate_authority_id,
    };
    act_on_certificate_authority_message("chat-room-query", record, channel, context).await
}

/// What `mirror_certificate_authority_message` keeps of each EscalateCertificateAuthority response.
#[derive(Clone, Debug, Default)]
pub struct CertificateAuthorityMessageRecord {
    pub message: String,
    pub parent_id: String,
    pub certificate_authority_id: String,
}

/// Where `mirror_certificate_authority_message` hands each record.
trait CertificateAuthorityMessageTarget {
    fn deliver(&self, record: CertificateAuthorityMessageRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CsrLifecycleService`.
struct CsrLifecycleSink {
    client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificateAuthorityMessageTarget for CsrLifecycleSink {
    fn deliver(&self, record: CertificateAuthorityMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::ExpireCsrRequest {
                    csr_id: record.certificate_authority_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.expire_csr(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.helpdesk.v1.TicketService`.
struct TicketSink {
    client: helpdesk::ticket_service_client::TicketServiceClient<Channel>,
    context: common::RequestContext,
}

impl CertificateAuthorityMessageTarget for TicketSink {
    fn deliver(&self, record: CertificateAuthorityMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(helpdesk::CreateTicketRequest {
                    request_id: record.certificate_authority_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.create_ticket(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.documents.v1.DocumentValidationService`.
struct DocumentValidationSink {
    client: documents::document_validation_service_client::DocumentValidationServiceClient<Channel>,
}

impl CertificateAuthorityMessageTarget for DocumentValidationSink {
    fn deliver(&self, record: CertificateAuthorityMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_document(documents::NormalizeDocumentRequest {
                    details: Some(documents::normalize_document_request::Details {
                        locale: record.certificate_authority_id,
                        ..Default::default()
                    }),
                    document: Some(documents::Document {
                        number: record.message,
                        ..Default::default()
                    }),
                    document_id: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
}

impl CertificateAuthorityMessageTarget for ChatRoomQuerySink {
    fn deliver(&self, record: CertificateAuthorityMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_chat_rooms(chat::FindChatRoomsRequest {
                    filter: Some(chat::find_chat_rooms_request::Filter {
                        owner_id: record.certificate_authority_id,
                        ..Default::default()
                    }),
                    order_by: record.parent_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_certificate_authority_message(
    kind: &str,
    record: CertificateAuthorityMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(CertificateAuthorityMessageRecord) -> Delivery>> = BTreeMap::new();
    let csr_lifecycle_channel = channel.clone();
    let csr_lifecycle_context = context.clone();
    actions.insert(
        "csr-lifecycle",
        Box::new(move |record| {
            CsrLifecycleSink {
                client: certificates::csr_lifecycle_service_client::CsrLifecycleServiceClient::new(
                    csr_lifecycle_channel.clone(),
                ),
                context: csr_lifecycle_context.clone(),
            }
            .deliver(record)
        }),
    );
    let ticket_channel = channel.clone();
    let ticket_context = context.clone();
    actions.insert(
        "ticket",
        Box::new(move |record| {
            TicketSink {
                client: helpdesk::ticket_service_client::TicketServiceClient::new(ticket_channel.clone()),
                context: ticket_context.clone(),
            }
            .deliver(record)
        }),
    );
    let document_validation_channel = channel.clone();
    actions.insert(
        "document-validation",
        Box::new(move |record| {
            DocumentValidationSink {
                client: documents::document_validation_service_client::DocumentValidationServiceClient::new(
                    document_validation_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    let chat_room_query_channel = channel.clone();
    actions.insert(
        "chat-room-query",
        Box::new(move |record| {
            ChatRoomQuerySink {
                client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(
                    chat_room_query_channel.clone(),
                ),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
