//! `sync_role_binding_export_number` reads
//! `acme.authorization.v1.RoleBindingExportService/CancelRoleBindingExport` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fanportal::v1 as fanportal;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_export_service_client::RoleBindingExportServiceClient::new(channel.clone());
    let loaded = source
        .cancel_role_binding_export(authorization::CancelRoleBindingExportRequest {
            context: Some(context.clone()),
            reason: "nightly-sync-role-binding-export-number".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let role_binding = loaded.role_binding.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = RoleBindingExportNumberRecord {
        number: role_binding.number,
        status: outcome.status,
    };
    let sinks: Vec<Box<dyn RoleBindingExportNumberWriter>> = vec![
        Box::new(FlagRolloutNotificationSink {
            client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(FanSessionQuerySink {
            client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        Box::new(FlagRolloutSyncSink {
            client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(channel),
        }),
    ];
    sinks[0].forward(record).await
}

/// What `sync_role_binding_export_number` keeps of each CancelRoleBindingExport response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingExportNumberRecord {
    pub number: String,
    pub status: String,
}

/// Where `sync_role_binding_export_number` hands each record.
trait RoleBindingExportNumberWriter {
    fn forward(&self, record: RoleBindingExportNumberRecord) -> Delivery;
}

/// Hands records to `acme.featureflags.v1.FlagRolloutNotificationService`.
struct FlagRolloutNotificationSink {
    client: featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient<Channel>,
}

impl RoleBindingExportNumberWriter for FlagRolloutNotificationSink {
    fn forward(&self, record: RoleBindingExportNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_flag_rollout_preferences(featureflags::GetFlagRolloutPreferencesRequest {
                    flag_rollout_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fanportal.v1.FanSessionQueryService`.
struct FanSessionQuerySink {
    client: fanportal::fan_session_query_service_client::FanSessionQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl RoleBindingExportNumberWriter for FanSessionQuerySink {
    fn forward(&self, record: RoleBindingExportNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fanportal::PeekFanSessionRequest {
                    fan_session_id: record.number,
                    ..Default::default()
                }),
                &context,
            )?;
            client.peek_fan_session(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.featureflags.v1.FlagRolloutSyncService`.
struct FlagRolloutSyncSink {
    client: featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient<Channel>,
}

impl RoleBindingExportNumberWriter for FlagRolloutSyncSink {
    fn forward(&self, record: RoleBindingExportNumberRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .push_flag_rollout(featureflags::PushFlagRolloutRequest {
                    flag_rollout_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
