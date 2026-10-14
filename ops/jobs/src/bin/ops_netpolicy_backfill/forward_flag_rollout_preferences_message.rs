//! `forward_flag_rollout_preferences_message` reads
//! `acme.featureflags.v1.FlagRolloutNotificationService/GetFlagRolloutPreferences` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::performers::v1 as performers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = featureflags::flag_rollout_notification_service_client::FlagRolloutNotificationServiceClient::new(
        channel.clone(),
    );
    let fetched = source
        .get_flag_rollout_preferences(featureflags::GetFlagRolloutPreferencesRequest {
            context: Some(context.clone()),
            flag_rollout_id: "current-forward-flag-rollout-preferences-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let flag_rollout = fetched.flag_rollout.unwrap_or_default();
    let record = FlagRolloutPreferencesMessageRecord {
        message: result.message,
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        flag_rollout_id: flag_rollout.flag_rollout_id,
        title: acme_kit::text::normalize(&flag_rollout.title),
    };
    dispatch_flag_rollout_preferences_message("crypto-key-query", record, channel, &context).await
}

/// What `forward_flag_rollout_preferences_message` keeps of each GetFlagRolloutPreferences response.
#[derive(Clone, Debug, Default)]
pub struct FlagRolloutPreferencesMessageRecord {
    pub message: String,
    pub correlation_id: String,
    pub flag_rollout_id: String,
    pub title: String,
}

/// Where `forward_flag_rollout_preferences_message` hands each record.
trait FlagRolloutPreferencesMessageTarget {
    fn write(&self, record: FlagRolloutPreferencesMessageRecord) -> Delivery;
}

/// Writes records to `acme.performers.v1.PerformerService`.
struct PerformerSink {
    client: performers::performer_service_client::PerformerServiceClient<Channel>,
}

impl FlagRolloutPreferencesMessageTarget for PerformerSink {
    fn write(&self, record: FlagRolloutPreferencesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_performer(performers::GetPerformerRequest {
                    performer_id: record.title,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl FlagRolloutPreferencesMessageTarget for ConfigSetLifecycleSink {
    fn write(&self, record: FlagRolloutPreferencesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .expire_config_set(configuration::ExpireConfigSetRequest {
                    config_set_id: record.title,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.CryptoKeyQueryService`.
struct CryptoKeyQuerySink {
    client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient<Channel>,
}

impl FlagRolloutPreferencesMessageTarget for CryptoKeyQuerySink {
    fn write(&self, record: FlagRolloutPreferencesMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_crypto_key(keymanagement::LookupCryptoKeyRequest {
                    crypto_key_id: record.title,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct FlagRolloutPreferencesMessageLogOnly;

impl FlagRolloutPreferencesMessageTarget for FlagRolloutPreferencesMessageLogOnly {
    fn write(&self, record: FlagRolloutPreferencesMessageRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.message);
            Ok(())
        })
    }
}

/// Picks the writer of a kind and runs it.
async fn dispatch_flag_rollout_preferences_message(
    kind: &str,
    record: FlagRolloutPreferencesMessageRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let performer = PerformerSink {
        client: performers::performer_service_client::PerformerServiceClient::new(channel.clone()),
    };
    let config_set_lifecycle = ConfigSetLifecycleSink {
        client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(
            channel.clone(),
        ),
    };
    let crypto_key_query = CryptoKeyQuerySink {
        client: keymanagement::crypto_key_query_service_client::CryptoKeyQueryServiceClient::new(channel),
    };
    let write: &dyn Fn(FlagRolloutPreferencesMessageRecord) -> Delivery = match kind {
        "performer" => &|record| performer.write(record),
        "config-set-lifecycle" => &|record| config_set_lifecycle.write(record),
        "crypto-key-query" => &|record| crypto_key_query.write(record),
        _ => &|record| FlagRolloutPreferencesMessageLogOnly.write(record),
    };
    write(record).await
}
