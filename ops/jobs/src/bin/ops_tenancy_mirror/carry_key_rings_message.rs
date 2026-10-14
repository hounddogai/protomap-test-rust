//! `carry_key_rings_message` reads `acme.keymanagement.v1.KeyRingCalculationService/RecalculateKeyRings` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_calculation_service_client::KeyRingCalculationServiceClient::new(channel.clone());
    let loaded = source
        .recalculate_key_rings(keymanagement::RecalculateKeyRingsRequest {
            context: Some(context.clone()),
            key_ring_id: "primary-carry-key-rings-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let breakdown = loaded.breakdown.unwrap_or_default();
    let key_ring = loaded.key_ring.unwrap_or_default();
    let record = KeyRingsMessageRecord {
        message: result.message,
        correlation_id: result.correlation_id,
        currency_code: acme_kit::text::normalize(&breakdown.currency_code),
        name: key_ring.name,
    };
    deliver_key_rings_message("permission-check-notification", record, channel, &context).await
}

/// What `carry_key_rings_message` keeps of each RecalculateKeyRings response.
#[derive(Clone, Debug, Default)]
pub struct KeyRingsMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub currency_code: String,
    pub name: String,
}

/// Where `carry_key_rings_message` hands each record.
trait KeyRingsMessageWriter {
    fn forward(&self, record: KeyRingsMessageRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRingApprovalService`.
struct KeyRingApprovalSink {
    client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient<Channel>,
}

impl KeyRingsMessageWriter for KeyRingApprovalSink {
    fn forward(&self, record: KeyRingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reject_key_ring(keymanagement::RejectKeyRingRequest {
                    key_ring: Some(keymanagement::KeyRing {
                        name: record.currency_code,
                        ..Default::default()
                    }),
                    key_ring_id: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.authorization.v1.PermissionCheckNotificationService`.
struct PermissionCheckNotificationSink {
    client:
        authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient<Channel>,
}

impl KeyRingsMessageWriter for PermissionCheckNotificationSink {
    fn forward(&self, record: KeyRingsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .notify_permission_check_watchers(authorization::NotifyPermissionCheckWatchersRequest {
                    permission_check_id: record.message,
                    reason: record.name,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct KeyRingsMessageLogOnly;

impl KeyRingsMessageWriter for KeyRingsMessageLogOnly {
    fn forward(&self, record: KeyRingsMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_key_rings_message(
    kind: &str,
    record: KeyRingsMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn KeyRingsMessageWriter> = match kind {
        "key-ring-approval" => Box::new(KeyRingApprovalSink { client: keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone()) }),
        "permission-check-notification" => Box::new(PermissionCheckNotificationSink { client: authorization::permission_check_notification_service_client::PermissionCheckNotificationServiceClient::new(channel) }),
        _ => Box::new(KeyRingsMessageLogOnly),
    };
    sink.forward(record).await
}
