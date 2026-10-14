//! `replay_config_set_report_id` reads `acme.configuration.v1.ConfigSetReportingService/ShareConfigSetReport`
//! and hands what it keeps to `acme.configuration.v1.EnvironmentValidationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_reporting_service_client::ConfigSetReportingServiceClient::new(channel.clone());
    let found = source
        .share_config_set_report(configuration::ShareConfigSetReportRequest {
            context: Some(context.clone()),
            config_set_id: "current-replay-config-set-report-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let config_set = found.config_set.unwrap_or_default();
    let record = ConfigSetReportIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        name: config_set.name,
        status: result.status,
    };
    EnvironmentValidationSink {
        client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient::new(channel),
        context: context.clone(),
    }
    .deliver(record)
    .await
}

/// What `replay_config_set_report_id` keeps of each ShareConfigSetReport response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetReportIdRecord {
    pub correlation_id: String,
    pub name: String,
    pub status: String,
}

/// Where `replay_config_set_report_id` hands each record.
trait ConfigSetReportIdSink {
    fn deliver(&self, record: ConfigSetReportIdRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.EnvironmentValidationService`.
struct EnvironmentValidationSink {
    client: configuration::environment_validation_service_client::EnvironmentValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetReportIdSink for EnvironmentValidationSink {
    fn deliver(&self, record: ConfigSetReportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::GetEnvironmentRulesRequest {
                    environment_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_environment_rules(request).await?;
            Ok(())
        })
    }
}
