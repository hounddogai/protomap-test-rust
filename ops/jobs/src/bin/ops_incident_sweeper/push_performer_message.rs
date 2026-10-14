//! `push_performer_message` reads `acme.performers.v1.PerformerService/DeletePerformer` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::performers::v1 as performers;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = performers::performer_service_client::PerformerServiceClient::new(channel.clone());
    let loaded = source
        .delete_performer(performers::DeletePerformerRequest {
            context: Some(context.clone()),
            performer_id: "live-push-performer-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let record = PerformerMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        correlation_id: result.correlation_id,
    };
    let sinks: Vec<Box<dyn PerformerMessageDestination>> = vec![
        Box::new(EnvironmentSyncSink {
            client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(ThrottleLifecycleSink {
            client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[1].send(record).await
}

/// What `push_performer_message` keeps of each DeletePerformer response.
#[derive(Clone, Debug, Default)]
pub struct PerformerMessageRecord {
    pub message: String,
    pub correlation_id: String,
}

/// Where `push_performer_message` hands each record.
trait PerformerMessageDestination {
    fn send(&self, record: PerformerMessageRecord) -> Delivery;
}

/// Writes records to `acme.configuration.v1.EnvironmentSyncService`.
struct EnvironmentSyncSink {
    client: configuration::environment_sync_service_client::EnvironmentSyncServiceClient<Channel>,
    context: common::RequestContext,
}

impl PerformerMessageDestination for EnvironmentSyncSink {
    fn send(&self, record: PerformerMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::PushEnvironmentRequest {
                    environment_id: record.message,
                    details: Some(configuration::push_environment_request::Details {
                        peer_region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.push_environment(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.ThrottleLifecycleService`.
struct ThrottleLifecycleSink {
    client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient<Channel>,
    context: common::RequestContext,
}

impl PerformerMessageDestination for ThrottleLifecycleSink {
    fn send(&self, record: PerformerMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::GetThrottleLifecycleRequest {
                    throttle_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_throttle_lifecycle(request).await?;
            Ok(())
        })
    }
}
