//! `push_chat_room_id` reads `acme.chat.v1.ChatRoomService/UpdateChatRoom` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::chat::v1 as chat;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = chat::chat_room_service_client::ChatRoomServiceClient::new(channel.clone());
    let fetched = source
        .update_chat_room(chat::UpdateChatRoomRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let chat_room = fetched.chat_room.unwrap_or_default();
    let record = ChatRoomIdRecord {
        correlation_id: result.correlation_id,
        name: chat_room.name,
        status: result.status,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ChatRoomIdWriter>> = BTreeMap::new();
    sinks.insert(
        "config-set-validation",
        Box::new(ConfigSetValidationSink {
            client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "certificate",
        Box::new(CertificateSink {
            client: certificates::certificate_service_client::CertificateServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "throttle-validation",
        Box::new(ThrottleValidationSink {
            client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let Some(sink) = sinks.get("throttle-validation") else {
        anyhow::bail!("no sink registered for throttle-validation");
    };
    sink.send(record).await
}

/// What `push_chat_room_id` keeps of each UpdateChatRoom response.
#[derive(Clone, Debug, Default)]
pub struct ChatRoomIdRecord {
    pub correlation_id: String,
    pub name: String,
    pub status: String,
}

/// Where `push_chat_room_id` hands each record.
trait ChatRoomIdWriter {
    fn send(&self, record: ChatRoomIdRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.ConfigSetValidationService`.
struct ConfigSetValidationSink {
    client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ChatRoomIdWriter for ConfigSetValidationSink {
    fn send(&self, record: ChatRoomIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::LintConfigSetsRequest {
                    details: Some(configuration::lint_config_sets_request::Details {
                        locale: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_config_sets(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CertificateService`.
struct CertificateSink {
    client: certificates::certificate_service_client::CertificateServiceClient<Channel>,
    context: common::RequestContext,
}

impl ChatRoomIdWriter for CertificateSink {
    fn send(&self, record: ChatRoomIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::SearchCertificatesRequest {
                    filter: Some(certificates::search_certificates_request::Filter {
                        region: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_certificates(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.ThrottleValidationService`.
struct ThrottleValidationSink {
    client: ratelimiting::throttle_validation_service_client::ThrottleValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ChatRoomIdWriter for ThrottleValidationSink {
    fn send(&self, record: ChatRoomIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ValidateThrottleRequest {
                    details: Some(ratelimiting::validate_throttle_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    throttle: Some(ratelimiting::Throttle {
                        correlation_id: record.name,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.validate_throttle(request).await?;
            Ok(())
        })
    }
}
