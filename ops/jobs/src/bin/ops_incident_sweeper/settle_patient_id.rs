//! `settle_patient_id` reads `acme.patients.v1.PatientCalculationService/CalculatePatient` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        patients::patient_calculation_service_client::PatientCalculationServiceClient::new(channel.clone());
    let reply = source
        .calculate_patient(patients::CalculatePatientRequest {
            context: Some(context.clone()),
            patient_id: "current-settle-patient-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let patient = reply.patient.unwrap_or_default();
    let breakdown = reply.breakdown.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = PatientIdRecord {
        patient_id: acme_kit::text::normalize(&patient.patient_id),
        currency_code: breakdown.currency_code,
        message: result.message,
        status: result.status,
    };
    let sinks: Vec<(&str, Box<dyn PatientIdSink>)> = vec![
        (
            "experiment-calculation",
            Box::new(ExperimentCalculationSink {
                client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                    channel.clone(),
                ),
            }),
        ),
        (
            "flag-rollout-query",
            Box::new(FlagRolloutQuerySink {
                client: featureflags::flag_rollout_query_service_client::FlagRolloutQueryServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "flag-rollout-query" {
            continue;
        }
        sink.write(record.clone()).await?;
    }
    Ok(())
}

/// What `settle_patient_id` keeps of each CalculatePatient response.
#[derive(Clone, Debug, Default)]
pub struct PatientIdRecord {
    pub patient_id: String,
    pub currency_code: String,
    pub message: String,
    pub status: String,
}

/// Where `settle_patient_id` hands each record.
trait PatientIdSink {
    fn write(&self, record: PatientIdRecord) -> Delivery;
}

/// Hands records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl PatientIdSink for ExperimentCalculationSink {
    fn write(&self, record: PatientIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .calculate_experiment(experiments::CalculateExperimentRequest {
                    experiment: Some(experiments::Experiment {
                        title: record.status,
                        ..Default::default()
                    }),
                    experiment_id: record.message,
                    reason: record.currency_code,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagRolloutQueryService`.
struct FlagRolloutQuerySink {
    client: featureflags::flag_rollout_query_service_client::FlagRolloutQueryServiceClient<Channel>,
}

impl PatientIdSink for FlagRolloutQuerySink {
    fn write(&self, record: PatientIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_flag_rollouts(featureflags::FindFlagRolloutsRequest {
                    filter: Some(featureflags::find_flag_rollouts_request::Filter {
                        query: record.currency_code,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
