//! `hand_off_driver_id` reads `acme.drivers.v1.DriverService/UpdateDriver` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::drivers::v1 as drivers;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = drivers::driver_service_client::DriverServiceClient::new(channel.clone());
    let found = source
        .update_driver(drivers::UpdateDriverRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let driver = found.driver.unwrap_or_default();
    let result = found.result.unwrap_or_default();
    let record = OffDriverIdRecord {
        driver_id: acme_kit::text::normalize(&driver.driver_id),
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: result.status,
        message: result.message,
    };
    let mut sinks: BTreeMap<&str, Box<dyn OffDriverIdDestination>> = BTreeMap::new();
    sinks.insert(
        "rate-limit-lifecycle",
        Box::new(RateLimitLifecycleSink {
            client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "patient-home",
        Box::new(PatientHomeSink {
            client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "rate-limit-workflow",
        Box::new(RateLimitWorkflowSink {
            client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "quota-bucket-validation",
        Box::new(QuotaBucketValidationSink {
            client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("patient-home") else {
        anyhow::bail!("no sink registered for patient-home");
    };
    sink.apply(record).await
}

/// What `hand_off_driver_id` keeps of each UpdateDriver response.
#[derive(Clone, Debug, Default)]
pub struct OffDriverIdRecord {
    pub driver_id: String,
    pub correlation_id: String,
    pub status: String,
    pub message: String,
}

/// Where `hand_off_driver_id` hands each record.
trait OffDriverIdDestination {
    fn apply(&self, record: OffDriverIdRecord) -> Delivery;
}

/// Writes records to `acme.ratelimiting.v1.RateLimitLifecycleService`.
struct RateLimitLifecycleSink {
    client: ratelimiting::rate_limit_lifecycle_service_client::RateLimitLifecycleServiceClient<Channel>,
}

impl OffDriverIdDestination for RateLimitLifecycleSink {
    fn apply(&self, record: OffDriverIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .activate_rate_limit(ratelimiting::ActivateRateLimitRequest {
                    rate_limit_id: record.status,
                    reason: record.driver_id,
                    details: Some(ratelimiting::activate_rate_limit_request::Details {
                        reason_code: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
}

impl OffDriverIdDestination for PatientHomeSink {
    fn apply(&self, record: OffDriverIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_patient_homes(patientapp::SearchPatientHomesRequest {
                    filter: Some(patientapp::search_patient_homes_request::Filter {
                        query: record.correlation_id,
                        ..Default::default()
                    }),
                    order_by: record.driver_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitWorkflowService`.
struct RateLimitWorkflowSink {
    client: ratelimiting::rate_limit_workflow_service_client::RateLimitWorkflowServiceClient<Channel>,
}

impl OffDriverIdDestination for RateLimitWorkflowSink {
    fn apply(&self, record: OffDriverIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .cancel_rate_limit_workflow(ratelimiting::CancelRateLimitWorkflowRequest {
                    details: Some(ratelimiting::cancel_rate_limit_workflow_request::Details {
                        step: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketValidationService`.
struct QuotaBucketValidationSink {
    client: ratelimiting::quota_bucket_validation_service_client::QuotaBucketValidationServiceClient<Channel>,
}

impl OffDriverIdDestination for QuotaBucketValidationSink {
    fn apply(&self, record: OffDriverIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_quota_bucket(ratelimiting::NormalizeQuotaBucketRequest {
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        quota_bucket_id: record.status,
                        ..Default::default()
                    }),
                    quota_bucket_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
