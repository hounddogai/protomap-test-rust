//! `forward_config_change_report_id` reads
//! `acme.configuration.v1.ConfigChangeReportingService/FetchConfigChangeReport` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient::new(channel.clone());
    let loaded = source
        .fetch_config_change_report(configuration::FetchConfigChangeReportRequest {
            context: Some(context.clone()),
            config_change_id: "live-forward-config-change-report-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let config_change = loaded.config_change.unwrap_or_default();
    let record = ConfigChangeReportIdRecord {
        correlation_id: result.correlation_id,
        source: config_change.source,
    };
    let mut sinks: BTreeMap<&str, Box<dyn ConfigChangeReportIdWriter>> = BTreeMap::new();
    sinks.insert(
        "csr-validation",
        Box::new(CsrValidationSink {
            client: certificates::csr_validation_service_client::CsrValidationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "csr-approval",
        Box::new(CsrApprovalSink {
            client: certificates::csr_approval_service_client::CsrApprovalServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let Some(sink) = sinks.get("csr-approval") else {
        anyhow::bail!("no sink registered for csr-approval");
    };
    sink.deliver(record).await
}

/// What `forward_config_change_report_id` keeps of each FetchConfigChangeReport response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeReportIdRecord {
    pub correlation_id: String,
    pub source: String,
}

/// Where `forward_config_change_report_id` hands each record.
trait ConfigChangeReportIdWriter {
    fn deliver(&self, record: ConfigChangeReportIdRecord) -> Delivery;
}

/// Sends records to `acme.certificates.v1.CsrValidationService`.
struct CsrValidationSink {
    client: certificates::csr_validation_service_client::CsrValidationServiceClient<Channel>,
}

impl ConfigChangeReportIdWriter for CsrValidationSink {
    fn deliver(&self, record: ConfigChangeReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .score_csr(certificates::ScoreCsrRequest {
                    csr_id: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.certificates.v1.CsrApprovalService`.
struct CsrApprovalSink {
    client: certificates::csr_approval_service_client::CsrApprovalServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigChangeReportIdWriter for CsrApprovalSink {
    fn deliver(&self, record: ConfigChangeReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(certificates::RejectCsrRequest {
                    details: Some(certificates::reject_csr_request::Details {
                        approver_email: record.correlation_id,
                        ..Default::default()
                    }),
                    csr: Some(certificates::Csr {
                        csr_id: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.reject_csr(request).await?;
            Ok(())
        })
    }
}
