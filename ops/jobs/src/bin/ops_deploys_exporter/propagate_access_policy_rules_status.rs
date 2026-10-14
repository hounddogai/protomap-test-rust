//! `propagate_access_policy_rules_status` reads
//! `acme.authorization.v1.AccessPolicyValidationService/GetAccessPolicyRules` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = authorization::access_policy_validation_service_client::AccessPolicyValidationServiceClient::new(
        channel.clone(),
    );
    let loaded = source
        .get_access_policy_rules(authorization::GetAccessPolicyRulesRequest {
            context: Some(context.clone()),
            access_policy_id: "default-propagate-access-policy-rules-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let access_policy = loaded.access_policy.unwrap_or_default();
    let record = AccessPolicyRulesStatusRecord {
        status: result.status,
        access_policy_id: access_policy.access_policy_id,
        description: acme_kit::text::normalize(&access_policy.description),
    };
    deliver_access_policy_rules_status("flag-rollout-admin", record, channel, &context).await
}

/// What `propagate_access_policy_rules_status` keeps of each GetAccessPolicyRules response.
#[derive(Clone, Debug, Default)]
pub struct AccessPolicyRulesStatusRecord {
    pub status: String,
    pub access_policy_id: String,
    pub description: String,
}

/// Where `propagate_access_policy_rules_status` hands each record.
trait AccessPolicyRulesStatusDestination {
    fn write(&self, record: AccessPolicyRulesStatusRecord) -> Delivery;
}

/// Writes records to `acme.fleet.v1.VehicleQueryService`.
struct VehicleQuerySink {
    client: fleet::vehicle_query_service_client::VehicleQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl AccessPolicyRulesStatusDestination for VehicleQuerySink {
    fn write(&self, record: AccessPolicyRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fleet::BrowseVehiclesRequest {
                    filter: Some(fleet::browse_vehicles_request::Filter {
                        region: record.description,
                        ..Default::default()
                    }),
                    page: Some(common::PageRequest {
                        page_token: record.status,
                        ..Default::default()
                    }),
                    order_by: record.access_policy_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.browse_vehicles(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.featureflags.v1.FlagRolloutAdminService`.
struct FlagRolloutAdminSink {
    client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient<Channel>,
    context: common::RequestContext,
}

impl AccessPolicyRulesStatusDestination for FlagRolloutAdminSink {
    fn write(&self, record: AccessPolicyRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(featureflags::RepairFlagRolloutRequest {
                    details: Some(featureflags::repair_flag_rollout_request::Details {
                        checksum: record.access_policy_id,
                        resolution: record.description,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.repair_flag_rollout(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct AccessPolicyRulesStatusLogOnly;

impl AccessPolicyRulesStatusDestination for AccessPolicyRulesStatusLogOnly {
    fn write(&self, record: AccessPolicyRulesStatusRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.status);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_access_policy_rules_status(
    kind: &str,
    record: AccessPolicyRulesStatusRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn AccessPolicyRulesStatusDestination> = match kind {
        "vehicle-query" => Box::new(VehicleQuerySink {
            client: fleet::vehicle_query_service_client::VehicleQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
        "flag-rollout-admin" => Box::new(FlagRolloutAdminSink {
            client: featureflags::flag_rollout_admin_service_client::FlagRolloutAdminServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(AccessPolicyRulesStatusLogOnly),
    };
    sink.write(record).await
}
