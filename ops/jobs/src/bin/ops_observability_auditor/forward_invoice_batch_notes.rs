//! `forward_invoice_batch_notes` reads `acme.invoicing.v1.InvoiceBatchService/UpdateInvoiceBatch` and hands
//! what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::experiments::v1 as experiments;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::invoicing::v1 as invoicing;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = invoicing::invoice_batch_service_client::InvoiceBatchServiceClient::new(channel.clone());
    let reply = source
        .update_invoice_batch(invoicing::UpdateInvoiceBatchRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let invoice_batch = reply.invoice_batch.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = InvoiceBatchNotesRecord {
        notes: invoice_batch.notes,
        status: result.status,
    };
    deliver_invoice_batch_notes("patient", record, channel, &context).await
}

/// What `forward_invoice_batch_notes` keeps of each UpdateInvoiceBatch response.
#[derive(Clone, Debug, Default)]
pub struct InvoiceBatchNotesRecord {
    pub notes: String,
    pub status: String,
}

/// Where `forward_invoice_batch_notes` hands each record.
trait InvoiceBatchNotesWriter {
    fn apply(&self, record: InvoiceBatchNotesRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
}

impl InvoiceBatchNotesWriter for FlagEvaluationNotificationSink {
    fn apply(&self, record: InvoiceBatchNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_evaluation_preferences(featureflags::GetFlagEvaluationPreferencesRequest {
                    flag_evaluation_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.patients.v1.PatientService`.
struct PatientSink {
    client: patients::patient_service_client::PatientServiceClient<Channel>,
}

impl InvoiceBatchNotesWriter for PatientSink {
    fn apply(&self, record: InvoiceBatchNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_patient(patients::CreatePatientRequest {
                    patient: Some(patients::Patient {
                        patient_id: record.notes,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.experiments.v1.ExperimentCalculationService`.
struct ExperimentCalculationSink {
    client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient<Channel>,
}

impl InvoiceBatchNotesWriter for ExperimentCalculationSink {
    fn apply(&self, record: InvoiceBatchNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_experiment(experiments::QuoteExperimentRequest {
                    experiment_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.ratelimiting.v1.RateLimitValidationService`.
struct RateLimitValidationSink {
    client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient<Channel>,
}

impl InvoiceBatchNotesWriter for RateLimitValidationSink {
    fn apply(&self, record: InvoiceBatchNotesRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .normalize_rate_limit(ratelimiting::NormalizeRateLimitRequest {
                    details: Some(ratelimiting::normalize_rate_limit_request::Details {
                        locale: record.notes,
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
struct InvoiceBatchNotesLogOnly;

impl InvoiceBatchNotesWriter for InvoiceBatchNotesLogOnly {
    fn apply(&self, record: InvoiceBatchNotesRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.notes);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_invoice_batch_notes(
    kind: &str,
    record: InvoiceBatchNotesRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn InvoiceBatchNotesWriter> = match kind {
        "flag-evaluation-notification" => Box::new(FlagEvaluationNotificationSink {
            client:
                featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(
                    channel.clone(),
                ),
        }),
        "patient" => Box::new(PatientSink {
            client: patients::patient_service_client::PatientServiceClient::new(channel.clone()),
        }),
        "experiment-calculation" => Box::new(ExperimentCalculationSink {
            client: experiments::experiment_calculation_service_client::ExperimentCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "rate-limit-validation" => Box::new(RateLimitValidationSink {
            client: ratelimiting::rate_limit_validation_service_client::RateLimitValidationServiceClient::new(channel),
        }),
        _ => Box::new(InvoiceBatchNotesLogOnly),
    };
    sink.apply(record).await
}
