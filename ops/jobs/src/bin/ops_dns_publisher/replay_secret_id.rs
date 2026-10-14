//! `replay_secret_id` reads `acme.secrets.v1.SecretService/UpdateSecret` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = secrets::secret_service_client::SecretServiceClient::new(channel.clone());
    let found = source
        .update_secret(secrets::UpdateSecretRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let secret = found.secret.unwrap_or_default();
    let record = SecretIdRecord {
        correlation_id: result.correlation_id,
        secret_id: acme_kit::text::normalize(&secret.secret_id),
        message: result.message,
    };
    deliver_secret_id("audit-event-query", record, channel, &context).await
}

/// What `replay_secret_id` keeps of each UpdateSecret response.
#[derive(Clone, Debug, Default)]
pub struct SecretIdRecord {
    pub correlation_id: String,
    pub secret_id: String,
    pub message: String,
}

/// Where `replay_secret_id` hands each record.
trait SecretIdDestination {
    fn send(&self, record: SecretIdRecord) -> Delivery;
}

/// Hands records to `acme.fanportal.v1.FanSessionService`.
struct FanSessionSink {
    client: fanportal::fan_session_service_client::FanSessionServiceClient<Channel>,
}

impl SecretIdDestination for FanSessionSink {
    fn send(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_fan_session(fanportal::UpdateFanSessionRequest {
                    fan_session: Some(fanportal::FanSession {
                        subject: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.edgegateway.v1.ApiKeyService`.
struct ApiKeySink {
    client: edgegateway::api_key_service_client::ApiKeyServiceClient<Channel>,
}

impl SecretIdDestination for ApiKeySink {
    fn send(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_api_key(edgegateway::UpdateApiKeyRequest {
                    api_key: Some(edgegateway::ApiKey {
                        api_key_id: record.secret_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl SecretIdDestination for ConfigSetLifecycleSink {
    fn send(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_config_set(configuration::RenewConfigSetRequest {
                    reason: record.correlation_id,
                    details: Some(configuration::renew_config_set_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    config_set_id: record.secret_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.audit.v1.AuditEventQueryService`.
struct AuditEventQuerySink {
    client: audit::audit_event_query_service_client::AuditEventQueryServiceClient<Channel>,
}

impl SecretIdDestination for AuditEventQuerySink {
    fn send(&self, record: SecretIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_audit_events(audit::BrowseAuditEventsRequest {
                    filter: Some(audit::browse_audit_events_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.secret_id,
                        ..Default::default()
                    }),
                    order_by: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct SecretIdLogOnly;

impl SecretIdDestination for SecretIdLogOnly {
    fn send(&self, record: SecretIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_secret_id(
    kind: &str,
    record: SecretIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SecretIdDestination> = match kind {
        "fan-session" => Box::new(FanSessionSink {
            client: fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone()),
        }),
        "api-key" => Box::new(ApiKeySink {
            client: edgegateway::api_key_service_client::ApiKeyServiceClient::new(channel.clone()),
        }),
        "config-set-lifecycle" => Box::new(ConfigSetLifecycleSink {
            client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
        "audit-event-query" => Box::new(AuditEventQuerySink {
            client: audit::audit_event_query_service_client::AuditEventQueryServiceClient::new(channel),
        }),
        _ => Box::new(SecretIdLogOnly),
    };
    sink.send(record).await
}
