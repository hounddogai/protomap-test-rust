//! `replay_access_policy_status` reads `acme.authorization.v1.AccessPolicyAdminService/RestoreAccessPolicy`
//! and hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::drivers::v1 as drivers;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::access_policy_admin_service_client::AccessPolicyAdminServiceClient::new(channel.clone());
    let found = source
        .restore_access_policy(authorization::RestoreAccessPolicyRequest {
            context: Some(context.clone()),
            reason: "nightly-replay-access-policy-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let access_policy = found.access_policy.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let record = AccessPolicyStatusRecord {
        status: result.status,
        description: access_policy.description,
        outcome_status: outcome.status,
    };
    let sinks: Vec<Box<dyn AccessPolicyStatusDestination>> = vec![
        Box::new(DriverSink {
            client: drivers::driver_service_client::DriverServiceClient::new(channel.clone()),
        }),
        Box::new(ConfigSetLifecycleSink {
            client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(channel),
        }),
    ];
    sinks[0].forward(record).await
}

/// What `replay_access_policy_status` keeps of each RestoreAccessPolicy response.
#[derive(Clone, Debug, Default)]
pub struct AccessPolicyStatusRecord {
    pub status: String,
    pub description: String,
    pub outcome_status: String,
}

/// Where `replay_access_policy_status` hands each record.
trait AccessPolicyStatusDestination {
    fn forward(&self, record: AccessPolicyStatusRecord) -> Delivery;
}

/// Passes records to `acme.drivers.v1.DriverService`.
struct DriverSink {
    client: drivers::driver_service_client::DriverServiceClient<Channel>,
}

impl AccessPolicyStatusDestination for DriverSink {
    fn forward(&self, record: AccessPolicyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .create_driver(drivers::CreateDriverRequest {
                    request_id: record.description,
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

impl AccessPolicyStatusDestination for ConfigSetLifecycleSink {
    fn forward(&self, record: AccessPolicyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .expire_config_set(configuration::ExpireConfigSetRequest {
                    details: Some(configuration::expire_config_set_request::Details {
                        reason_code: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
