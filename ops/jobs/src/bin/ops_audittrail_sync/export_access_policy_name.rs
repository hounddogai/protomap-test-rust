//! `export_access_policy_name` reads
//! `acme.authorization.v1.AccessPolicyValidationService/ValidateAccessPolicy` and hands what it keeps to
//! `acme.featureflags.v1.FlagRolloutCalculationService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
        channel.clone(),
    );
    let found = source
        .validate_access_policy(authorization::ValidateAccessPolicyRequest {
            context: Some(context.clone()),
            reason: "main-export-access-policy-name".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let access_policy = found.access_policy.unwrap_or_default();
    let record = AccessPolicyNameRecord {
        name: access_policy.name,
        description: access_policy.description,
    };
    FlagRolloutCalculationSink {
        client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
            channel,
        ),
    }
    .write(record)
    .await
}

/// What `export_access_policy_name` keeps of each ValidateAccessPolicy response.
#[derive(Clone, Debug, Default)]
pub struct AccessPolicyNameRecord {
    pub name: String,
    pub description: String,
}

/// Where `export_access_policy_name` hands each record.
trait AccessPolicyNameTarget {
    fn write(&self, record: AccessPolicyNameRecord) -> Delivery;
}

/// Writes records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
}

impl AccessPolicyNameTarget for FlagRolloutCalculationSink {
    fn write(&self, record: AccessPolicyNameRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_flag_rollout_options(featureflags::CompareFlagRolloutOptionsRequest {
                    details: Some(featureflags::compare_flag_rollout_options_request::Details {
                        currency_code: record.description,
                        ..Default::default()
                    }),
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.name,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
