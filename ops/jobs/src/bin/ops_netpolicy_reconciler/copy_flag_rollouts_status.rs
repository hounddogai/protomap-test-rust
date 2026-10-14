//! `copy_flag_rollouts_status` reads `acme.featureflags.v1.FlagRolloutSyncService/SyncFlagRollouts` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::ledger::v1 as ledger;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::flag_rollout_sync_service_client::FlagRolloutSyncServiceClient::new(channel.clone());
    let reply = source
        .sync_flag_rollouts(featureflags::SyncFlagRolloutsRequest {
            context: Some(context.clone()),
            reason: "primary-copy-flag-rollouts-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let outcome = reply.outcome.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = FlagRolloutsStatusRecord {
        status: outcome.status,
        correlation_id: result.correlation_id,
    };
    let mut sinks: BTreeMap<&str, Box<dyn FlagRolloutsStatusSink>> = BTreeMap::new();
    sinks.insert(
        "ledger-account-query",
        Box::new(LedgerAccountQuerySink {
            client: ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "config-set-lifecycle",
        Box::new(ConfigSetLifecycleSink {
            client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "feature-flag-admin",
        Box::new(FeatureFlagAdminSink {
            client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("ledger-account-query") else {
        anyhow::bail!("no sink registered for ledger-account-query");
    };
    sink.send(record).await
}

/// What `copy_flag_rollouts_status` keeps of each SyncFlagRollouts response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutsStatusRecord {
    pub status: String,
    pub correlation_id: String,
}

/// Where `copy_flag_rollouts_status` hands each record.
trait FlagRolloutsStatusSink {
    fn send(&self, record: FlagRolloutsStatusRecord) -> Delivery;
}

/// Hands records to `acme.ledger.v1.LedgerAccountQueryService`.
struct LedgerAccountQuerySink {
    client: ledger::ledger_account_query_service_client::LedgerAccountQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagRolloutsStatusSink for LedgerAccountQuerySink {
    fn send(&self, record: FlagRolloutsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ledger::LookupLedgerAccountRequest {
                    ledger_account_id: record.correlation_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_ledger_account(request).await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl FlagRolloutsStatusSink for ConfigSetLifecycleSink {
    fn send(&self, record: FlagRolloutsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .terminate_config_set(configuration::TerminateConfigSetRequest {
                    details: Some(configuration::terminate_config_set_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FeatureFlagAdminService`.
struct FeatureFlagAdminSink {
    client: featureflags::feature_flag_admin_service_client::FeatureFlagAdminServiceClient<Channel>,
}

impl FlagRolloutsStatusSink for FeatureFlagAdminSink {
    fn send(&self, record: FlagRolloutsStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .transfer_feature_flag(featureflags::TransferFeatureFlagRequest {
                    reason: record.correlation_id,
                    details: Some(featureflags::transfer_feature_flag_request::Details {
                        reason_code: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
