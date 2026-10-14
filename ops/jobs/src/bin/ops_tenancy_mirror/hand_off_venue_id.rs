//! `hand_off_venue_id` reads `acme.venues.v1.VenueQueryService/PeekVenue` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = venues::venue_query_service_client::VenueQueryServiceClient::new(channel.clone());
    let loaded = source
        .peek_venue(venues::PeekVenueRequest {
            context: Some(context.clone()),
            venue_id: "live-hand-off-venue-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let venue = loaded.venue.unwrap_or_default();
    let record = OffVenueIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        name: venue.name,
    };
    let sinks: Vec<(&str, Box<dyn OffVenueIdWriter>)> = vec![
        ("chat-room-query", Box::new(ChatRoomQuerySink { client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(channel.clone()) })),
        ("venue-query", Box::new(VenueQuerySink { client: venues::venue_query_service_client::VenueQueryServiceClient::new(channel.clone()) })),
        ("certificate-authority-approval", Box::new(CertificateAuthorityApprovalSink { client: certificates::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient::new(channel) })),
    ];
    for (kind, sink) in &sinks {
        if *kind != "certificate-authority-approval" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `hand_off_venue_id` keeps of each PeekVenue response.
#[derive(Clone, Debug, Default)]
pub struct OffVenueIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub name: String,
}

/// Where `hand_off_venue_id` hands each record.
trait OffVenueIdWriter {
    fn push(&self, record: OffVenueIdRecord) -> Delivery;
}

/// Sends records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
}

impl OffVenueIdWriter for ChatRoomQuerySink {
    fn push(&self, record: OffVenueIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_chat_room(chat::LookupChatRoomRequest {
                    chat_room_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.venues.v1.VenueQueryService`.
struct VenueQuerySink {
    client: venues::venue_query_service_client::VenueQueryServiceClient<Channel>,
}

impl OffVenueIdWriter for VenueQuerySink {
    fn push(&self, record: OffVenueIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_venue(venues::LookupVenueRequest {
                    venue_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateAuthorityApprovalService`.
struct CertificateAuthorityApprovalSink {
    client:
        certificates::certificate_authority_approval_service_client::CertificateAuthorityApprovalServiceClient<Channel>,
}

impl OffVenueIdWriter for CertificateAuthorityApprovalSink {
    fn push(&self, record: OffVenueIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_certificate_authority(certificates::ApproveCertificateAuthorityRequest {
                    details: Some(certificates::approve_certificate_authority_request::Details {
                        approver_email: record.correlation_id,
                        ..Default::default()
                    }),
                    certificate_authority: Some(certificates::CertificateAuthority {
                        parent_id: record.status,
                        certificate_authority_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
