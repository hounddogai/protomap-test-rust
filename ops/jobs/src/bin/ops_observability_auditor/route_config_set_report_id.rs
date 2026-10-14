//! `route_config_set_report_id` reads `acme.configuration.v1.ConfigSetReportingService/DeleteConfigSetReport`
//! and hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fuel::v1 as fuel;
use acme_proto::acme::secrets::v1 as secrets;
use acme_proto::acme::shows::v1 as shows;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_reporting_service_client::ConfigSetReportingServiceClient::new(channel.clone());
    let found = source
        .delete_config_set_report(configuration::DeleteConfigSetReportRequest {
            context: Some(context.clone()),
            config_set_id: "nightly-route-config-set-report-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let record = ConfigSetReportIdRecord {
        correlation_id: result.correlation_id,
        status: acme_kit::text::normalize(&result.status),
        message: result.message,
    };
    act_on_config_set_report_id("fuel-card-query", record, channel, context).await
}

/// What `route_config_set_report_id` keeps of each DeleteConfigSetReport response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetReportIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub message: String,
}

/// Where `route_config_set_report_id` hands each record.
trait ConfigSetReportIdTarget {
    fn push(&self, record: ConfigSetReportIdRecord) -> Delivery;
}

/// Writes records to `acme.secrets.v1.SecretVersionApprovalService`.
struct SecretVersionApprovalSink {
    client: secrets::secret_version_approval_service_client::SecretVersionApprovalServiceClient<Channel>,
}

impl ConfigSetReportIdTarget for SecretVersionApprovalSink {
    fn push(&self, record: ConfigSetReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .approve_secret_version(secrets::ApproveSecretVersionRequest {
                    secret_version: Some(secrets::SecretVersion {
                        secret_version_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.shows.v1.ShowService`.
struct ShowSink {
    client: shows::show_service_client::ShowServiceClient<Channel>,
}

impl ConfigSetReportIdTarget for ShowSink {
    fn push(&self, record: ConfigSetReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_show(shows::CreateShowRequest {
                    request_id: record.message,
                    show: Some(shows::Show {
                        title: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.certificates.v1.CertificateWorkflowService`.
struct CertificateWorkflowSink {
    client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient<Channel>,
}

impl ConfigSetReportIdTarget for CertificateWorkflowSink {
    fn push(&self, record: ConfigSetReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .advance_certificate_workflow(certificates::AdvanceCertificateWorkflowRequest {
                    certificate: Some(certificates::Certificate {
                        notes: record.message,
                        certificate_id: record.status,
                        ..Default::default()
                    }),
                    details: Some(certificates::advance_certificate_workflow_request::Details {
                        initiated_by: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.fuel.v1.FuelCardQueryService`.
struct FuelCardQuerySink {
    client: fuel::fuel_card_query_service_client::FuelCardQueryServiceClient<Channel>,
}

impl ConfigSetReportIdTarget for FuelCardQuerySink {
    fn push(&self, record: ConfigSetReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_fuel_card(fuel::LookupFuelCardRequest {
                    fuel_card_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_config_set_report_id(
    kind: &str,
    record: ConfigSetReportIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(ConfigSetReportIdRecord) -> Delivery>> = BTreeMap::new();
    let secret_version_approval_channel = channel.clone();
    actions.insert(
        "secret-version-approval",
        Box::new(move |record| {
            SecretVersionApprovalSink {
                client: secrets::secret_version_approval_service_client::SecretVersionApprovalServiceClient::new(
                    secret_version_approval_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    let show_channel = channel.clone();
    actions.insert(
        "show",
        Box::new(move |record| {
            ShowSink {
                client: shows::show_service_client::ShowServiceClient::new(show_channel.clone()),
            }
            .push(record)
        }),
    );
    let certificate_workflow_channel = channel.clone();
    actions.insert(
        "certificate-workflow",
        Box::new(move |record| {
            CertificateWorkflowSink {
                client: certificates::certificate_workflow_service_client::CertificateWorkflowServiceClient::new(
                    certificate_workflow_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    let fuel_card_query_channel = channel.clone();
    actions.insert(
        "fuel-card-query",
        Box::new(move |record| {
            FuelCardQuerySink {
                client: fuel::fuel_card_query_service_client::FuelCardQueryServiceClient::new(
                    fuel_card_query_channel.clone(),
                ),
            }
            .push(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
