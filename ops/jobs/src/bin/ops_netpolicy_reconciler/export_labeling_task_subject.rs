//! `export_labeling_task_subject` reads `acme.labeling.v1.LabelingTaskService/UpdateLabelingTask` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::labeling::v1 as labeling;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = labeling::labeling_task_service_client::LabelingTaskServiceClient::new(channel.clone());
    let fetched = source
        .update_labeling_task(labeling::UpdateLabelingTaskRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let labeling_task = fetched.labeling_task.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = LabelingTaskSubjectRecord {
        subject: acme_kit::text::normalize(&labeling_task.subject),
        correlation_id: result.correlation_id,
    };
    dispatch_labeling_task_subject("patient-calculation", record, channel, &context).await
}

/// What `export_labeling_task_subject` keeps of each UpdateLabelingTask response.
#[derive(Clone, Debug, Default)]
pub struct LabelingTaskSubjectRecord {
    pub subject: String,
    pub correlation_id: String,
}

/// Where `export_labeling_task_subject` hands each record.
trait LabelingTaskSubjectSink {
    fn write(&self, record: LabelingTaskSubjectRecord) -> Delivery;
}

/// Writes records to `acme.patients.v1.PatientCalculationService`.
struct PatientCalculationSink {
    client: patients::patient_calculation_service_client::PatientCalculationServiceClient<Channel>,
}

impl LabelingTaskSubjectSink for PatientCalculationSink {
    fn write(&self, record: LabelingTaskSubjectRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_patient(patients::EstimatePatientRequest {
                    details: Some(patients::estimate_patient_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.careplans.v1.CarePlanCalculationService`.
struct CarePlanCalculationSink {
    client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient<Channel>,
}

impl LabelingTaskSubjectSink for CarePlanCalculationSink {
    fn write(&self, record: LabelingTaskSubjectRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_care_plan(careplans::EstimateCarePlanRequest {
                    reason: record.correlation_id,
                    care_plan: Some(careplans::CarePlan {
                        care_plan_id: record.subject,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.ThrottleLifecycleService`.
struct ThrottleLifecycleSink {
    client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient<Channel>,
}

impl LabelingTaskSubjectSink for ThrottleLifecycleSink {
    fn write(&self, record: LabelingTaskSubjectRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .terminate_throttle(ratelimiting::TerminateThrottleRequest {
                    details: Some(ratelimiting::terminate_throttle_request::Details {
                        reason_code: record.correlation_id,
                        ..Default::default()
                    }),
                    throttle_id: record.subject,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct LabelingTaskSubjectLogOnly;

impl LabelingTaskSubjectSink for LabelingTaskSubjectLogOnly {
    fn write(&self, record: LabelingTaskSubjectRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.subject);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_labeling_task_subject(
    kind: &str,
    record: LabelingTaskSubjectRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let patient_calculation = PatientCalculationSink {
        client: patients::patient_calculation_service_client::PatientCalculationServiceClient::new(channel.clone()),
    };
    let care_plan_calculation = CarePlanCalculationSink {
        client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient::new(channel.clone()),
    };
    let throttle_lifecycle = ThrottleLifecycleSink {
        client: ratelimiting::throttle_lifecycle_service_client::ThrottleLifecycleServiceClient::new(channel),
    };
    let write: &dyn Fn(LabelingTaskSubjectRecord) -> Delivery = match kind {
        "patient-calculation" => &|record| patient_calculation.write(record),
        "care-plan-calculation" => &|record| care_plan_calculation.write(record),
        "throttle-lifecycle" => &|record| throttle_lifecycle.write(record),
        _ => &|record| LabelingTaskSubjectLogOnly.write(record),
    };
    write(record).await
}
