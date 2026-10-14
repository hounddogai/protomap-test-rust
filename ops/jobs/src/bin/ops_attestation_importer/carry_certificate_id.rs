//! `carry_certificate_id` reads `acme.certificates.v1.CertificateService/UpdateCertificate` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::employees::v1 as employees;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::certificate_service_client::CertificateServiceClient::new(channel.clone());
    let loaded = source
        .update_certificate(certificates::UpdateCertificateRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let certificate = loaded.certificate.unwrap_or_default();
    let result = loaded.result.unwrap_or_default();
    let record = CertificateIdRecord {
        certificate_id: certificate.certificate_id,
        notes: certificate.notes,
        message: acme_kit::text::normalize(&result.message),
        number: certificate.number,
    };
    let mut sinks: BTreeMap<String, Box<dyn CertificateIdDestination>> = BTreeMap::new();
    sinks.insert(
        "venue-validation".to_owned(),
        Box::new(VenueValidationSink {
            client: venues::venue_validation_service_client::VenueValidationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "quota-bucket-query".to_owned(),
        Box::new(QuotaBucketQuerySink {
            client: ratelimiting::quota_bucket_query_service_client::QuotaBucketQueryServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "employee".to_owned(),
        Box::new(EmployeeSink {
            client: employees::employee_service_client::EmployeeServiceClient::new(channel),
        }),
    );
    let target = std::env::var("CARRY_CERTIFICATE_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.forward(record).await,
        None => Ok(()),
    }
}

/// What `carry_certificate_id` keeps of each UpdateCertificate response.
#[derive(Clone, Debug, Default)]
pub struct CertificateIdRecord {
    pub certificate_id: String,
    pub notes: String,
    pub message: String,
    pub number: String,
}

/// Where `carry_certificate_id` hands each record.
trait CertificateIdDestination {
    fn forward(&self, record: CertificateIdRecord) -> Delivery;
}

/// Writes records to `acme.venues.v1.VenueValidationService`.
struct VenueValidationSink {
    client: venues::venue_validation_service_client::VenueValidationServiceClient<Channel>,
}

impl CertificateIdDestination for VenueValidationSink {
    fn forward(&self, record: CertificateIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .preview_venue(venues::PreviewVenueRequest {
                    reason: record.notes,
                    venue: Some(venues::Venue {
                        code: record.number,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.QuotaBucketQueryService`.
struct QuotaBucketQuerySink {
    client: ratelimiting::quota_bucket_query_service_client::QuotaBucketQueryServiceClient<Channel>,
}

impl CertificateIdDestination for QuotaBucketQuerySink {
    fn forward(&self, record: CertificateIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .peek_quota_bucket(ratelimiting::PeekQuotaBucketRequest {
                    quota_bucket_id: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.employees.v1.EmployeeService`.
struct EmployeeSink {
    client: employees::employee_service_client::EmployeeServiceClient<Channel>,
}

impl CertificateIdDestination for EmployeeSink {
    fn forward(&self, record: CertificateIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_employee(employees::UpdateEmployeeRequest {
                    employee: Some(employees::Employee {
                        employee_id: record.certificate_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
