//! `push_flag_evaluation_id` reads `acme.featureflags.v1.FlagEvaluationAdminService/ArchiveFlagEvaluation`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::appointments::v1 as appointments;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(channel.clone());
    let fetched = source
        .archive_flag_evaluation(featureflags::ArchiveFlagEvaluationRequest {
            context: Some(context.clone()),
            reason: "default-push-flag-evaluation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let flag_evaluation = fetched.flag_evaluation.unwrap_or_default();
    let record = FlagEvaluationIdRecord {
        flag_evaluation_id: acme_kit::text::normalize(&flag_evaluation.flag_evaluation_id),
        kind: flag_evaluation.kind,
    };
    dispatch_flag_evaluation_id("appointment-query", record, channel, &context).await
}

/// What `push_flag_evaluation_id` keeps of each ArchiveFlagEvaluation response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationIdRecord {
    pub flag_evaluation_id: String,
    pub kind: String,
}

/// Where `push_flag_evaluation_id` hands each record.
trait FlagEvaluationIdWriter {
    fn apply(&self, record: FlagEvaluationIdRecord) -> Delivery;
}

/// Passes records to `acme.appointments.v1.AppointmentQueryService`.
struct AppointmentQuerySink {
    client: appointments::appointment_query_service_client::AppointmentQueryServiceClient<Channel>,
}

impl FlagEvaluationIdWriter for AppointmentQuerySink {
    fn apply(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_appointments(appointments::FindAppointmentsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.QuotaBucketCalculationService`.
struct QuotaBucketCalculationSink {
    client: ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient<Channel>,
}

impl FlagEvaluationIdWriter for QuotaBucketCalculationSink {
    fn apply(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_quota_bucket(ratelimiting::EstimateQuotaBucketRequest {
                    details: Some(ratelimiting::estimate_quota_bucket_request::Details {
                        currency_code: record.kind,
                        ..Default::default()
                    }),
                    quota_bucket: Some(ratelimiting::QuotaBucket {
                        currency_code: record.flag_evaluation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityValidationService`.
struct CertificateAuthorityValidationSink {
    client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient<
        Channel,
    >,
}

impl FlagEvaluationIdWriter for CertificateAuthorityValidationSink {
    fn apply(&self, record: FlagEvaluationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_certificate_authority(certificates::PreviewCertificateAuthorityRequest {
                    details: Some(certificates::preview_certificate_authority_request::Details {
                        locale: record.kind,
                        ..Default::default()
                    }),
                    certificate_authority: Some(certificates::CertificateAuthority {
                        certificate_authority_id: record.flag_evaluation_id,
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
struct FlagEvaluationIdLogOnly;

impl FlagEvaluationIdWriter for FlagEvaluationIdLogOnly {
    fn apply(&self, record: FlagEvaluationIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.flag_evaluation_id);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_flag_evaluation_id(
    kind: &str,
    record: FlagEvaluationIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let appointment_query = AppointmentQuerySink {
        client: appointments::appointment_query_service_client::AppointmentQueryServiceClient::new(channel.clone()),
    };
    let quota_bucket_calculation = QuotaBucketCalculationSink {
        client: ratelimiting::quota_bucket_calculation_service_client::QuotaBucketCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let certificate_authority_validation = CertificateAuthorityValidationSink { client: certificates::certificate_authority_validation_service_client::CertificateAuthorityValidationServiceClient::new(channel) };
    let write: &dyn Fn(FlagEvaluationIdRecord) -> Delivery = match kind {
        "appointment-query" => &|record| appointment_query.apply(record),
        "quota-bucket-calculation" => &|record| quota_bucket_calculation.apply(record),
        "certificate-authority-validation" => &|record| certificate_authority_validation.apply(record),
        _ => &|record| FlagEvaluationIdLogOnly.apply(record),
    };
    write(record).await
}
