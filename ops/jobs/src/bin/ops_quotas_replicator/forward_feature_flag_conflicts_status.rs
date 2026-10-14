//! `forward_feature_flag_conflicts_status` reads
//! `acme.featureflags.v1.FeatureFlagSyncService/ListFeatureFlagConflicts` and hands what it keeps to the sink
//! of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::itsm::v1 as itsm;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::feature_flag_sync_service_client::FeatureFlagSyncServiceClient::new(channel.clone());
    let found = source
        .list_feature_flag_conflicts(featureflags::ListFeatureFlagConflictsRequest {
            context: Some(context.clone()),
            order_by: "live-forward-feature-flag-conflicts-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let page_info = found.page_info.unwrap_or_default();
    let record = FeatureFlagConflictsStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        message: acme_kit::text::normalize(&result.message),
        next_page_token: page_info.next_page_token,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    deliver_feature_flag_conflicts_status("patient-home", record, channel, &context).await
}

/// What `forward_feature_flag_conflicts_status` keeps of each ListFeatureFlagConflicts response.
#[derive(Clone, Debug, Default)]
pub struct FeatureFlagConflictsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `forward_feature_flag_conflicts_status` hands each record.
trait FeatureFlagConflictsStatusWriter {
    fn push(&self, record: FeatureFlagConflictsStatusRecord) -> Delivery;
}

/// Sends records to `acme.itsm.v1.IncidentService`.
struct IncidentSink {
    client: itsm::incident_service_client::IncidentServiceClient<Channel>,
}

impl FeatureFlagConflictsStatusWriter for IncidentSink {
    fn push(&self, record: FeatureFlagConflictsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_incidents(itsm::ListIncidentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.correlation_id,
                        ..Default::default()
                    }),
                    filter: Some(itsm::list_incidents_request::Filter {
                        owner_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
}

impl FeatureFlagConflictsStatusWriter for PatientHomeSink {
    fn push(&self, record: FeatureFlagConflictsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_patient_home(patientapp::UpdatePatientHomeRequest {
                    patient_home: Some(patientapp::PatientHome {
                        subject: record.message,
                        patient_home_id: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.careplans.v1.CarePlanCalculationService`.
struct CarePlanCalculationSink {
    client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl FeatureFlagConflictsStatusWriter for CarePlanCalculationSink {
    fn push(&self, record: FeatureFlagConflictsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(careplans::SimulateCarePlanRequest {
                    care_plan: Some(careplans::CarePlan {
                        notes: record.status,
                        care_plan_id: record.next_page_token,
                        ..Default::default()
                    }),
                    reason: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.simulate_care_plan(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct FeatureFlagConflictsStatusLogOnly;

impl FeatureFlagConflictsStatusWriter for FeatureFlagConflictsStatusLogOnly {
    fn push(&self, record: FeatureFlagConflictsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_feature_flag_conflicts_status(
    kind: &str,
    record: FeatureFlagConflictsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn FeatureFlagConflictsStatusWriter> = match kind {
        "incident" => Box::new(IncidentSink {
            client: itsm::incident_service_client::IncidentServiceClient::new(channel.clone()),
        }),
        "patient-home" => Box::new(PatientHomeSink {
            client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel.clone()),
        }),
        "care-plan-calculation" => Box::new(CarePlanCalculationSink {
            client: careplans::care_plan_calculation_service_client::CarePlanCalculationServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(FeatureFlagConflictsStatusLogOnly),
    };
    sink.push(record).await
}
