//! `replay_csrs_subject` reads `acme.certificates.v1.CsrValidationService/LintCsrs` and hands what it keeps
//! to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = certificates::csr_validation_service_client::CsrValidationServiceClient::new(channel.clone());
    let found = source
        .lint_csrs(certificates::LintCsrsRequest {
            context: Some(context.clone()),
            csr_id: "live-replay-csrs-subject".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let csr = found.csr.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = CsrsSubjectRecord {
        subject: acme_kit::text::normalize(&csr.subject),
        status: outcome.status,
        csr_id: csr.csr_id,
    };
    let sinks: Vec<Box<dyn CsrsSubjectTarget>> = vec![
        Box::new(FlagEvaluationExportSink {
            client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(FeatureFlagQuerySink {
            client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(ConfigSetCalculationSink {
            client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(
                channel,
            ),
        }),
    ];
    sinks[2].send(record).await
}

/// What `replay_csrs_subject` keeps of each LintCsrs response.
#[derive(Clone, Debug, Default)]
pub struct CsrsSubjectRecord {
    pub subject: String,
    pub status: String,
    pub csr_id: String,
}

/// Where `replay_csrs_subject` hands each record.
trait CsrsSubjectTarget {
    fn send(&self, record: CsrsSubjectRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationExportService`.
struct FlagEvaluationExportSink {
    client: featureflags::flag_evaluation_export_service_client::FlagEvaluationExportServiceClient<Channel>,
}

impl CsrsSubjectTarget for FlagEvaluationExportSink {
    fn send(&self, record: CsrsSubjectRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .schedule_flag_evaluation_export(featureflags::ScheduleFlagEvaluationExportRequest {
                    flag_evaluation_id: record.subject,
                    details: Some(featureflags::schedule_flag_evaluation_export_request::Details {
                        format: record.csr_id,
                        cron: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FeatureFlagQueryService`.
struct FeatureFlagQuerySink {
    client: featureflags::feature_flag_query_service_client::FeatureFlagQueryServiceClient<Channel>,
}

impl CsrsSubjectTarget for FeatureFlagQuerySink {
    fn send(&self, record: CsrsSubjectRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_feature_flags(featureflags::SuggestFeatureFlagsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.csr_id,
                        ..Default::default()
                    }),
                    filter: Some(featureflags::suggest_feature_flags_request::Filter {
                        region: record.status,
                        query: record.subject,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
}

impl CsrsSubjectTarget for ConfigSetCalculationSink {
    fn send(&self, record: CsrsSubjectRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .quote_config_set(configuration::QuoteConfigSetRequest {
                    config_set: Some(configuration::ConfigSet {
                        parent_id: record.subject,
                        name: record.csr_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
