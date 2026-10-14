//! `carry_patients_message` reads `acme.patients.v1.PatientQueryService/SummarizePatients` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::routing::v1 as routing;
use acme_proto::acme::secrets::v1 as secrets;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = patients::patient_query_service_client::PatientQueryServiceClient::new(channel.clone());
    let found = source
        .summarize_patients(patients::SummarizePatientsRequest {
            context: Some(context.clone()),
            reason: "main-carry-patients-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let period = found.period.unwrap_or_default();
    let record = PatientsMessageRecord {
        message: result.message,
        report_id: acme_kit::text::normalize(&found.report_id),
        download_uri: acme_kit::text::normalize(&found.download_uri),
        timezone: period.timezone,
    };
    let mut sinks: BTreeMap<&str, Box<dyn PatientsMessageDestination>> = BTreeMap::new();
    sinks.insert(
        "certificate-query",
        Box::new(CertificateQuerySink {
            client: certificates::certificate_query_service_client::CertificateQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "route-query",
        Box::new(RouteQuerySink {
            client: routing::route_query_service_client::RouteQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "access-grant-validation",
        Box::new(AccessGrantValidationSink {
            client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("access-grant-validation") else {
        anyhow::bail!("no sink registered for access-grant-validation");
    };
    sink.apply(record).await
}

/// What `carry_patients_message` keeps of each SummarizePatients response.
#[derive(Clone, Debug, Default)]
pub struct PatientsMessageRecord {
    pub message: String,
    pub report_id: String,
    pub download_uri: String,
    pub timezone: String,
}

/// Where `carry_patients_message` hands each record.
trait PatientsMessageDestination {
    fn apply(&self, record: PatientsMessageRecord) -> Delivery;
}

/// Hands records to `acme.certificates.v1.CertificateQueryService`.
struct CertificateQuerySink {
    client: certificates::certificate_query_service_client::CertificateQueryServiceClient<Channel>,
}

impl PatientsMessageDestination for CertificateQuerySink {
    fn apply(&self, record: PatientsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_certificates(certificates::SuggestCertificatesRequest {
                    order_by: record.download_uri,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.routing.v1.RouteQueryService`.
struct RouteQuerySink {
    client: routing::route_query_service_client::RouteQueryServiceClient<Channel>,
}

impl PatientsMessageDestination for RouteQuerySink {
    fn apply(&self, record: PatientsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_routes(routing::SummarizeRoutesRequest {
                    details: Some(routing::summarize_routes_request::Details {
                        cron: record.timezone,
                        format: record.download_uri,
                        ..Default::default()
                    }),
                    route_id: record.report_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.secrets.v1.AccessGrantValidationService`.
struct AccessGrantValidationSink {
    client: secrets::access_grant_validation_service_client::AccessGrantValidationServiceClient<Channel>,
}

impl PatientsMessageDestination for AccessGrantValidationSink {
    fn apply(&self, record: PatientsMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_access_grant_decision(secrets::ExplainAccessGrantDecisionRequest {
                    access_grant_id: record.download_uri,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
