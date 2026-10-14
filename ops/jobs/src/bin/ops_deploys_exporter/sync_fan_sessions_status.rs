//! `sync_fan_sessions_status` reads `acme.fanportal.v1.FanSessionService/SearchFanSessions` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = fanportal::fan_session_service_client::FanSessionServiceClient::new(channel.clone());
    let reply = source
        .search_fan_sessions(fanportal::SearchFanSessionsRequest {
            context: Some(context.clone()),
            order_by: "current-sync-fan-sessions-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let page_info = reply.page_info.unwrap_or_default();
    let record = FanSessionsStatusRecord {
        status: result.status,
        message: acme_kit::text::normalize(&result.message),
        next_page_token: page_info.next_page_token,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
    };
    dispatch_fan_sessions_status("flag-evaluation-calculation", record, channel, &context).await
}

/// What `sync_fan_sessions_status` keeps of each SearchFanSessions response.
#[derive(Clone, Debug, Default)]
pub struct FanSessionsStatusRecord {
    pub status: String,
    pub message: String,
    pub next_page_token: String,
    pub correlation_id: String,
}

/// Where `sync_fan_sessions_status` hands each record.
trait FanSessionsStatusDestination {
    fn deliver(&self, record: FanSessionsStatusRecord) -> Delivery;
}

/// Passes records to `acme.authorization.v1.PermissionCheckAdminService`.
struct PermissionCheckAdminSink {
    client: authorization::permission_check_admin_service_client::PermissionCheckAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl FanSessionsStatusDestination for PermissionCheckAdminSink {
    fn deliver(&self, record: FanSessionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::PurgePermissionCheckRequest {
                    permission_check_id: record.next_page_token,
                    etag: record.correlation_id,
                    reason: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.purge_permission_check(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.configuration.v1.ConfigChangeCalculationService`.
struct ConfigChangeCalculationSink {
    client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient<Channel>,
}

impl FanSessionsStatusDestination for ConfigChangeCalculationSink {
    fn deliver(&self, record: FanSessionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .explain_config_change_calculation(configuration::ExplainConfigChangeCalculationRequest {
                    details: Some(configuration::explain_config_change_calculation_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagEvaluationCalculationService`.
struct FlagEvaluationCalculationSink {
    client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient<Channel>,
}

impl FanSessionsStatusDestination for FlagEvaluationCalculationSink {
    fn deliver(&self, record: FanSessionsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_flag_evaluation(featureflags::EstimateFlagEvaluationRequest {
                    flag_evaluation_id: record.message,
                    flag_evaluation: Some(featureflags::FlagEvaluation {
                        correlation_id: record.next_page_token,
                        flag_evaluation_id: record.correlation_id,
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
struct FanSessionsStatusLogOnly;

impl FanSessionsStatusDestination for FanSessionsStatusLogOnly {
    fn deliver(&self, record: FanSessionsStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_fan_sessions_status(
    kind: &str,
    record: FanSessionsStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let permission_check_admin = PermissionCheckAdminSink {
        client: authorization::permission_check_admin_service_client::PermissionCheckAdminServiceClient::new(
            channel.clone(),
        ),
        context: context.clone(),
    };
    let config_change_calculation = ConfigChangeCalculationSink {
        client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
            channel.clone(),
        ),
    };
    let flag_evaluation_calculation = FlagEvaluationCalculationSink {
        client: featureflags::flag_evaluation_calculation_service_client::FlagEvaluationCalculationServiceClient::new(
            channel,
        ),
    };
    let write: &dyn Fn(FanSessionsStatusRecord) -> Delivery = match kind {
        "permission-check-admin" => &|record| permission_check_admin.deliver(record),
        "config-change-calculation" => &|record| config_change_calculation.deliver(record),
        "flag-evaluation-calculation" => &|record| flag_evaluation_calculation.deliver(record),
        _ => &|record| FanSessionsStatusLogOnly.deliver(record),
    };
    write(record).await
}
