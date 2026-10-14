//! `settle_environment_report_name` reads
//! `acme.configuration.v1.EnvironmentReportingService/FetchEnvironmentReport` and hands what it keeps to
//! `acme.patients.v1.PatientQueryService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::environment_reporting_service_client::EnvironmentReportingServiceClient::new(channel.clone());
    let reply = source
        .fetch_environment_report(configuration::FetchEnvironmentReportRequest {
            context: Some(context.clone()),
            environment_id: "primary-settle-environment-report-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let environment = reply.environment.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = EnvironmentReportNameRecord {
        name: environment.name,
        status: result.status,
    };
    PatientQuerySink {
        client: patients::patient_query_service_client::PatientQueryServiceClient::new(channel),
        context: context.clone(),
    }
    .apply(record)
    .await
}

/// What `settle_environment_report_name` keeps of each FetchEnvironmentReport response.
#[derive(Clone, Debug, Default)]
pub struct EnvironmentReportNameRecord {
    pub name: String,
    pub status: String,
}

/// Where `settle_environment_report_name` hands each record.
trait EnvironmentReportNameWriter {
    fn apply(&self, record: EnvironmentReportNameRecord) -> Delivery;
}

/// Passes records to `acme.patients.v1.PatientQueryService`.
struct PatientQuerySink {
    client: patients::patient_query_service_client::PatientQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl EnvironmentReportNameWriter for PatientQuerySink {
    fn apply(&self, record: EnvironmentReportNameRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patients::CountPatientsRequest {
                    order_by: record.status,
                    filter: Some(patients::count_patients_request::Filter {
                        owner_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.count_patients(request).await?;
            Ok(())
        })
    }
}
