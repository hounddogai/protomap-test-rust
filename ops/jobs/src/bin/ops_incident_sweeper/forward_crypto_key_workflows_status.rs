//! `forward_crypto_key_workflows_status` reads
//! `acme.keymanagement.v1.CryptoKeyWorkflowService/ListCryptoKeyWorkflows` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::crypto_key_workflow_service_client::CryptoKeyWorkflowServiceClient::new(channel.clone());
    let reply = source
        .list_crypto_key_workflows(keymanagement::ListCryptoKeyWorkflowsRequest {
            context: Some(context.clone()),
            order_by: "default-forward-crypto-key-workflows-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = CryptoKeyWorkflowsStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        next_page_token: page_info.next_page_token,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    deliver_crypto_key_workflows_status("access-grant-workflow", record, channel, &context).await
}

/// What `forward_crypto_key_workflows_status` keeps of each ListCryptoKeyWorkflows response.
#[derive(Clone, Debug, Default)]
pub struct CryptoKeyWorkflowsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `forward_crypto_key_workflows_status` hands each record.
trait CryptoKeyWorkflowsStatusSink {
    fn send(&self, record: CryptoKeyWorkflowsStatusRecord) -> Delivery;
}

/// Passes records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl CryptoKeyWorkflowsStatusSink for RateLimitValidationSink {
    fn send(&self, record: CryptoKeyWorkflowsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_rate_limit(ratelimiting::PreviewRateLimitRequest {
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.secrets.v1.AccessGrantWorkflowService`.
struct AccessGrantWorkflowSink {
    client: secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl CryptoKeyWorkflowsStatusSink for AccessGrantWorkflowSink {
    fn send(&self, record: CryptoKeyWorkflowsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(secrets::AdvanceAccessGrantWorkflowRequest {
                    details: Some(secrets::advance_access_grant_workflow_request::Details {
                        step: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.advance_access_grant_workflow(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRingCalculationService`.
struct KeyRingCalculationSink {
    client: keymanagement::key_ring_calculation_service_client::KeyRingCalculationServiceClient<Channel>,
}

impl CryptoKeyWorkflowsStatusSink for KeyRingCalculationSink {
    fn send(&self, record: CryptoKeyWorkflowsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_key_ring_calculation(keymanagement::ExplainKeyRingCalculationRequest {
                    key_ring: Some(keymanagement::KeyRing {
                        key_ring_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct CryptoKeyWorkflowsStatusLogOnly;

impl CryptoKeyWorkflowsStatusSink for CryptoKeyWorkflowsStatusLogOnly {
    fn send(&self, record: CryptoKeyWorkflowsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_crypto_key_workflows_status(
    kind: &str,
    record: CryptoKeyWorkflowsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn CryptoKeyWorkflowsStatusSink> = match kind {
        "rate-limit-validation" => Box::new(RateLimitValidationSink {
            client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        "access-grant-workflow" => Box::new(AccessGrantWorkflowSink {
            client: secrets::access_grant_workflow_service_client::AccessGrantWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "key-ring-calculation" => Box::new(KeyRingCalculationSink {
            client: keymanagement::key_ring_calculation_service_client::KeyRingCalculationServiceClient::new(channel),
        }),
        _ => Box::new(CryptoKeyWorkflowsStatusLogOnly),
    };
    sink.send(record).await
}
