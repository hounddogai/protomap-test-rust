//! `relay_get_feature_flags_id` reads `acme.featureflags.v1.FeatureFlagService/BatchGetFeatureFlags` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::itsm::v1 as itsm;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::feature_flag_service_client::FeatureFlagServiceClient::new(channel.clone());
    let loaded = source
        .batch_get_feature_flags(featureflags::BatchGetFeatureFlagsRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = GetFeatureFlagsIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    let sinks: Vec<(&str, Box<dyn GetFeatureFlagsIdSink>)> = vec![
        ("incident", Box::new(IncidentSink { client: itsm::incident_service_client::IncidentServiceClient::new(channel.clone()), context: context.clone() })),
        ("certificate-authority-validation", Box::new(CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel.clone()), context: context.clone() })),
        ("chat-room-query", Box::new(ChatRoomQuerySink { client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient::new(channel) })),
    ];
    for (kind, sink) in &sinks {
        if *kind != "chat-room-query" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `relay_get_feature_flags_id` keeps of each BatchGetFeatureFlags response.
#[derive(Clone, Debug, Default)]
pub struct GetFeatureFlagsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `relay_get_feature_flags_id` hands each record.
trait GetFeatureFlagsIdSink {
    fn push(&self, record: GetFeatureFlagsIdRecord) -> Delivery;
}

/// Sends records to `acme.itsm.v1.IncidentService`.
struct IncidentSink {
    client: itsm::incident_service_client::IncidentServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetFeatureFlagsIdSink for IncidentSink {
    fn push(&self, record: GetFeatureFlagsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(itsm::SearchIncidentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    order_by: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_incidents(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
    context: common::RequestContext,
}

impl GetFeatureFlagsIdSink for CertificateAuthorityValidationSink {
    fn push(&self, record: GetFeatureFlagsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::GetCertificateAuthorityRulesRequest {
                    certificate_authority_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_certificate_authority_rules(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.chat.v1.ChatRoomQueryService`.
struct ChatRoomQuerySink {
    client: chat::chat_room_query_service_client::ChatRoomQueryServiceClient<Channel>,
}

impl GetFeatureFlagsIdSink for ChatRoomQuerySink {
    fn push(&self, record: GetFeatureFlagsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_chat_rooms(chat::BrowseChatRoomsRequest {
                    order_by: record.status,
                    filter: Some(chat::browse_chat_rooms_request::Filter {
                        region: record.next_page_token,
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
