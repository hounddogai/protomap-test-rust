//! `settle_quota_bucket_options_status` reads
//! `acme.ratelimiting.v1.QuotaBucketCalculationService/CompareQuotaBucketOptions` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::sellers::v1 as sellers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .compare_quota_bucket_options(ratelimiting::CompareQuotaBucketOptionsRequest {
            context: Some(context.clone()),
            reason: "current-settle-quota-bucket-options-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let quota_bucket = loaded.quota_bucket.unwrap_or_default();
    let record = QuotaBucketOptionsStatusRecord {
        status: result.status,
        quota_bucket_id: quota_bucket.quota_bucket_id,
    };
    act_on_quota_bucket_options_status("key-rotation-workflow", record, channel, context).await
}

/// What `settle_quota_bucket_options_status` keeps of each CompareQuotaBucketOptions response.
#[derive(Clone, Debug, Default)]
pub struct QuotaBucketOptionsStatusRecord {
    pub status: String,
    pub quota_bucket_id: String,
}

/// Where `settle_quota_bucket_options_status` hands each record.
trait QuotaBucketOptionsStatusTarget {
    fn deliver(&self, record: QuotaBucketOptionsStatusRecord) -> Delivery;
}

/// Passes records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl QuotaBucketOptionsStatusTarget for KeyRotationWorkflowSink {
    fn deliver(&self, record: QuotaBucketOptionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(keymanagement::AdvanceKeyRotationWorkflowRequest {
                    key_rotation_id: record.quota_bucket_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.advance_key_rotation_workflow(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.sellers.v1.SellerService`.
struct SellerSink {
    client: sellers::seller_service_client::SellerServiceClient<Channel>,
}

impl QuotaBucketOptionsStatusTarget for SellerSink {
    fn deliver(&self, record: QuotaBucketOptionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .delete_seller(sellers::DeleteSellerRequest {
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_quota_bucket_options_status(
    kind: &str,
    record: QuotaBucketOptionsStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(QuotaBucketOptionsStatusRecord) -> Delivery>> = BTreeMap::new();
    let key_rotation_workflow_channel = channel.clone();
    let key_rotation_workflow_context = context.clone();
    actions.insert(
        "key-rotation-workflow",
        Box::new(move |record| {
            KeyRotationWorkflowSink {
                client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(
                    key_rotation_workflow_channel.clone(),
                ),
                context: key_rotation_workflow_context.clone(),
            }
            .deliver(record)
        }),
    );
    let seller_channel = channel.clone();
    actions.insert(
        "seller",
        Box::new(move |record| {
            SellerSink {
                client: sellers::seller_service_client::SellerServiceClient::new(seller_channel.clone()),
            }
            .deliver(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
