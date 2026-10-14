//! `forward_sellers_id` reads `acme.sellers.v1.SellerService/ListSellers` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = sellers::seller_service_client::SellerServiceClient::new(channel.clone());
    let loaded = source
        .list_sellers(sellers::ListSellersRequest {
            context: Some(context.clone()),
            order_by: "nightly-forward-sellers-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let page_info = loaded.page_info.unwrap_or_default();
    let record = SellersIdRecord {
        correlation_id: result.correlation_id,
        next_page_token: page_info.next_page_token,
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
    };
    deliver_sellers_id("environment-approval", record, channel, &context).await
}

/// What `forward_sellers_id` keeps of each ListSellers response.
#[derive(Clone, Debug, Default)]
pub struct SellersIdRecord {
    pub correlation_id: String,
    pub next_page_token: String,
    pub status: String,
    pub message: String,
}

/// Where `forward_sellers_id` hands each record.
trait SellersIdDestination {
    fn apply(&self, record: SellersIdRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.EnvironmentApprovalService`.
struct EnvironmentApprovalSink {
    client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl SellersIdDestination for EnvironmentApprovalSink {
    fn apply(&self, record: SellersIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::EscalateEnvironmentRequest {
                    environment: Some(configuration::Environment {
                        code: record.next_page_token,
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(configuration::escalate_environment_request::Details {
                        comment: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.escalate_environment(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.keymanagement.v1.KeyRotationValidationService`.
struct KeyRotationValidationSink {
    client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl SellersIdDestination for KeyRotationValidationSink {
    fn apply(&self, record: SellersIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::LintKeyRotationsRequest {
                    details: Some(keymanagement::lint_key_rotations_request::Details {
                        locale: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.lint_key_rotations(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct SellersIdLogOnly;

impl SellersIdDestination for SellersIdLogOnly {
    fn apply(&self, record: SellersIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_sellers_id(
    kind: &str,
    record: SellersIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn SellersIdDestination> = match kind {
        "environment-approval" => Box::new(EnvironmentApprovalSink {
            client: configuration::environment_approval_service_client::EnvironmentApprovalServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        "key-rotation-validation" => Box::new(KeyRotationValidationSink {
            client: keymanagement::key_rotation_validation_service_client::KeyRotationValidationServiceClient::new(
                channel,
            ),
            context: context.clone(),
        }),
        _ => Box::new(SellersIdLogOnly),
    };
    sink.apply(record).await
}
