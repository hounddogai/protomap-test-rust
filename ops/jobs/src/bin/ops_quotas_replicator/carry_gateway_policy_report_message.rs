//! `carry_gateway_policy_report_message` reads
//! `acme.edgegateway.v1.GatewayPolicyReportingService/FetchGatewayPolicyReport` and hands what it keeps to
//! the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_reporting_service_client::GatewayPolicyReportingServiceClient::new(channel.clone());
    let reply = source
        .fetch_gateway_policy_report(edgegateway::FetchGatewayPolicyReportRequest {
            context: Some(context.clone()),
            gateway_policy_id: "main-carry-gateway-policy-report-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let gateway_policy = reply.gateway_policy.unwrap_or_default();
    let record = GatewayPolicyReportMessageRecord {
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        name: gateway_policy.name,
    };
    let mut sinks: BTreeMap<&str, Box<dyn GatewayPolicyReportMessageTarget>> = BTreeMap::new();
    sinks.insert(
        "concession-stand-calculation",
        Box::new(ConcessionStandCalculationSink {
            client:
                concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient::new(
                    channel.clone(),
                ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "flag-evaluation-notification",
        Box::new(FlagEvaluationNotificationSink {
            client:
                featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "config-change-reporting",
        Box::new(ConfigChangeReportingSink {
            client: configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("flag-evaluation-notification") else {
        anyhow::bail!("no sink registered for flag-evaluation-notification");
    };
    sink.push(record).await
}

/// What `carry_gateway_policy_report_message` keeps of each FetchGatewayPolicyReport response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyReportMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub name: String,
}

/// Where `carry_gateway_policy_report_message` hands each record.
trait GatewayPolicyReportMessageTarget {
    fn push(&self, record: GatewayPolicyReportMessageRecord) -> Delivery;
}

/// Sends records to `acme.concessions.v1.ConcessionStandCalculationService`.
struct ConcessionStandCalculationSink {
    client: concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl GatewayPolicyReportMessageTarget for ConcessionStandCalculationSink {
    fn push(&self, record: GatewayPolicyReportMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(concessions::EstimateConcessionStandRequest {
                    details: Some(concessions::estimate_concession_stand_request::Details {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    concession_stand_id: record.name,
                    concession_stand: Some(concessions::ConcessionStand {
                        concession_stand_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.estimate_concession_stand(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationNotificationService`.
struct FlagEvaluationNotificationSink {
    client: featureflags::flag_evaluation_notification_service_client::FlagEvaluationNotificationServiceClient<Channel>,
}

impl GatewayPolicyReportMessageTarget for FlagEvaluationNotificationSink {
    fn push(&self, record: GatewayPolicyReportMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .unsubscribe_flag_evaluation_updates(featureflags::UnsubscribeFlagEvaluationUpdatesRequest {
                    reason: record.name,
                    details: Some(featureflags::unsubscribe_flag_evaluation_updates_request::Details {
                        template_id: record.correlation_id,
                        ..Default::default()
                    }),
                    flag_evaluation_id: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigChangeReportingService`.
struct ConfigChangeReportingSink {
    client: configuration::config_change_reporting_service_client::ConfigChangeReportingServiceClient<Channel>,
}

impl GatewayPolicyReportMessageTarget for ConfigChangeReportingSink {
    fn push(&self, record: GatewayPolicyReportMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_config_change_reports(configuration::ListConfigChangeReportsRequest {
                    filter: Some(configuration::list_config_change_reports_request::Filter {
                        query: record.message,
                        owner_id: record.name,
                        ..Default::default()
                    }),
                    order_by: record.correlation_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
