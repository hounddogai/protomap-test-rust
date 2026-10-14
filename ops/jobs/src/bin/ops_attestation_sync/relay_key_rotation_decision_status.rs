//! `relay_key_rotation_decision_status` reads
//! `acme.keymanagement.v1.KeyRotationValidationService/ExplainKeyRotationDecision` and hands what it keeps to
//! `acme.keymanagement.v1.KeyRingLifecycleService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient::new(channel.clone());
    let fetched = source
        .explain_key_rotation_decision(keymanagement::ExplainKeyRotationDecisionRequest {
            context: Some(context.clone()),
            reason: "nightly-relay-key-rotation-decision-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = fetched.outcome.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = KeyRotationDecisionStatusRecord {
        status: outcome.status,
        reference_id: outcome.reference_id,
        message: acme_kit::text::normalize(&result.message),
        outcome_message: outcome.message,
    };
    KeyRingLifecycleSink {
        client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient::new(channel),
    }
    .forward(record)
    .await
}

/// What `relay_key_rotation_decision_status` keeps of each ExplainKeyRotationDecision response.
#[derive(Clone, Debug, Default)]
pub struct KeyRotationDecisionStatusRecord {
    pub status: String,
    pub reference_id: String,
    pub message: String,
    pub outcome_message: String,
}

/// Where `relay_key_rotation_decision_status` hands each record.
trait KeyRotationDecisionStatusSink {
    fn forward(&self, record: KeyRotationDecisionStatusRecord) -> Delivery;
}

/// Hands records to `acme.keymanagement.v1.KeyRingLifecycleService`.
struct KeyRingLifecycleSink {
    client: keymanagement::key_ring_lifecycle_service_client::KeyRingLifecycleServiceClient<Channel>,
}

impl KeyRotationDecisionStatusSink for KeyRingLifecycleSink {
    fn forward(&self, record: KeyRotationDecisionStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_key_ring(keymanagement::RenewKeyRingRequest {
                    key_ring_id: record.message,
                    reason: record.reference_id,
                    details: Some(keymanagement::renew_key_ring_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
