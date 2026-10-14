//! `hand_off_quota_bucket_workflow_id` reads
//! `acme.ratelimiting.v1.QuotaBucketWorkflowService/StartQuotaBucketWorkflow` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::shows::v1 as shows;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        ratelimiting::quota_bucket_workflow_service_client::QuotaBucketWorkflowServiceClient::new(channel.clone());
    let reply = source
        .start_quota_bucket_workflow(ratelimiting::StartQuotaBucketWorkflowRequest {
            context: Some(context.clone()),
            reason: "current-hand-off-quota-bucket-workflow-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let quota_bucket = reply.quota_bucket.unwrap_or_default();
    let outcome = reply.outcome.unwrap_or_default();
    let record = OffQuotaBucketWorkflowIdRecord {
        correlation_id: result.correlation_id,
        currency_code: quota_bucket.currency_code,
        message: outcome.message,
    };
    dispatch_off_quota_bucket_workflow_id("show-calculation", record, channel, &context).await
}

/// What `hand_off_quota_bucket_workflow_id` keeps of each StartQuotaBucketWorkflow response.
#[derive(Clone, Debug, Default)]
pub struct OffQuotaBucketWorkflowIdRecord {
    pub correlation_id: String,
    pub currency_code: String,
    pub message: String,
}

/// Where `hand_off_quota_bucket_workflow_id` hands each record.
trait OffQuotaBucketWorkflowIdSink {
    fn forward(&self, record: OffQuotaBucketWorkflowIdRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.EnvironmentCalculationService`.
struct EnvironmentCalculationSink {
    client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient<Channel>,
}

impl OffQuotaBucketWorkflowIdSink for EnvironmentCalculationSink {
    fn forward(&self, record: OffQuotaBucketWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_environment(configuration::SimulateEnvironmentRequest {
                    reason: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.keymanagement.v1.KeyRingWorkflowService`.
struct KeyRingWorkflowSink {
    client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient<Channel>,
}

impl OffQuotaBucketWorkflowIdSink for KeyRingWorkflowSink {
    fn forward(&self, record: OffQuotaBucketWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .resume_key_ring_workflow(keymanagement::ResumeKeyRingWorkflowRequest {
                    key_ring_id: record.correlation_id,
                    key_ring: Some(keymanagement::KeyRing {
                        key_ring_id: record.message,
                        ..Default::default()
                    }),
                    reason: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.yards.v1.YardCalculationService`.
struct YardCalculationSink {
    client: yards::yard_calculation_service_client::YardCalculationServiceClient<Channel>,
}

impl OffQuotaBucketWorkflowIdSink for YardCalculationSink {
    fn forward(&self, record: OffQuotaBucketWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_yard_options(yards::CompareYardOptionsRequest {
                    yard: Some(yards::Yard {
                        code: record.currency_code,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.shows.v1.ShowCalculationService`.
struct ShowCalculationSink {
    client: shows::show_calculation_service_client::ShowCalculationServiceClient<Channel>,
}

impl OffQuotaBucketWorkflowIdSink for ShowCalculationSink {
    fn forward(&self, record: OffQuotaBucketWorkflowIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_show_rates(shows::GetShowRatesRequest {
                    show_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OffQuotaBucketWorkflowIdLogOnly;

impl OffQuotaBucketWorkflowIdSink for OffQuotaBucketWorkflowIdLogOnly {
    fn forward(&self, record: OffQuotaBucketWorkflowIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.correlation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_off_quota_bucket_workflow_id(
    kind: &str,
    record: OffQuotaBucketWorkflowIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let environment_calculation = EnvironmentCalculationSink {
        client: configuration::environment_calculation_service_client::EnvironmentCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let key_ring_workflow = KeyRingWorkflowSink {
        client: keymanagement::key_ring_workflow_service_client::KeyRingWorkflowServiceClient::new(channel.clone()),
    };
    let yard_calculation = YardCalculationSink {
        client: yards::yard_calculation_service_client::YardCalculationServiceClient::new(channel.clone()),
    };
    let show_calculation = ShowCalculationSink {
        client: shows::show_calculation_service_client::ShowCalculationServiceClient::new(channel),
    };
    let write: &dyn Fn(OffQuotaBucketWorkflowIdRecord) -> Delivery = match kind {
        "environment-calculation" => &|record| environment_calculation.forward(record),
        "key-ring-workflow" => &|record| key_ring_workflow.forward(record),
        "yard-calculation" => &|record| yard_calculation.forward(record),
        "show-calculation" => &|record| show_calculation.forward(record),
        _ => &|record| OffQuotaBucketWorkflowIdLogOnly.forward(record),
    };
    write(record).await
}
