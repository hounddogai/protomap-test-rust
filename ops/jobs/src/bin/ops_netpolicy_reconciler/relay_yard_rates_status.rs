//! `relay_yard_rates_status` reads `acme.yards.v1.YardCalculationService/GetYardRates` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::yards::v1 as yards;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = yards::yard_calculation_service_client::YardCalculationServiceClient::new(channel.clone());
    let loaded = source
        .get_yard_rates(yards::GetYardRatesRequest {
            context: Some(context.clone()),
            yard_id: "current-relay-yard-rates-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let yard = loaded.yard.unwrap_or_default();
    let record = YardRatesStatusRecord {
        status: result.status,
        message: result.message,
        yard_id: yard.yard_id,
        code: yard.code,
    };
    act_on_yard_rates_status("throttle-sync", record, channel, context).await
}

/// What `relay_yard_rates_status` keeps of each GetYardRates response.
#[derive(Clone, Debug, Default)]
pub struct YardRatesStatusRecord {
    pub status: String,
    pub message: String,
    pub yard_id: String,
    pub code: String,
}

/// Where `relay_yard_rates_status` hands each record.
trait YardRatesStatusDestination {
    fn write(&self, record: YardRatesStatusRecord) -> Delivery;
}

/// Writes records to `acme.authorization.v1.RoleBindingExportService`.
struct RoleBindingExportSink {
    client: authorization::role_binding_export_service_client::RoleBindingExportServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardRatesStatusDestination for RoleBindingExportSink {
    fn write(&self, record: YardRatesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::ListRoleBindingExportsRequest {
                    filter: Some(authorization::list_role_binding_exports_request::Filter {
                        query: record.code,
                        region: record.status,
                        owner_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.list_role_binding_exports(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.identity.v1.UserValidationService`.
struct UserValidationSink {
    client: identity::user_validation_service_client::UserValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardRatesStatusDestination for UserValidationSink {
    fn write(&self, record: YardRatesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(identity::ValidateUserRequest {
                    user: Some(identity::User {
                        user_id: record.code,
                        ..Default::default()
                    }),
                    details: Some(identity::validate_user_request::Details {
                        locale: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.validate_user(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.RoleBindingValidationService`.
struct RoleBindingValidationSink {
    client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl YardRatesStatusDestination for RoleBindingValidationSink {
    fn write(&self, record: YardRatesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(authorization::CheckRoleBindingEligibilityRequest {
                    role_binding_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.check_role_binding_eligibility(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.ThrottleSyncService`.
struct ThrottleSyncSink {
    client: ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient<Channel>,
}

impl YardRatesStatusDestination for ThrottleSyncSink {
    fn write(&self, record: YardRatesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .pull_throttles(ratelimiting::PullThrottlesRequest {
                    details: Some(ratelimiting::pull_throttles_request::Details {
                        peer_region: record.code,
                        cursor: record.yard_id,
                        ..Default::default()
                    }),
                    reason: record.message,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_yard_rates_status(
    kind: &str,
    record: YardRatesStatusRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(YardRatesStatusRecord) -> Delivery>> = BTreeMap::new();
    let role_binding_export_channel = channel.clone();
    let role_binding_export_context = context.clone();
    actions.insert(
        "role-binding-export",
        Box::new(move |record| {
            RoleBindingExportSink {
                client: authorization::role_binding_export_service_client::RoleBindingExportServiceClient::new(
                    role_binding_export_channel.clone(),
                ),
                context: role_binding_export_context.clone(),
            }
            .write(record)
        }),
    );
    let user_validation_channel = channel.clone();
    let user_validation_context = context.clone();
    actions.insert(
        "user-validation",
        Box::new(move |record| {
            UserValidationSink {
                client: identity::user_validation_service_client::UserValidationServiceClient::new(
                    user_validation_channel.clone(),
                ),
                context: user_validation_context.clone(),
            }
            .write(record)
        }),
    );
    let role_binding_validation_channel = channel.clone();
    let role_binding_validation_context = context.clone();
    actions.insert(
        "role-binding-validation",
        Box::new(move |record| {
            RoleBindingValidationSink {
                client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient::new(
                    role_binding_validation_channel.clone(),
                ),
                context: role_binding_validation_context.clone(),
            }
            .write(record)
        }),
    );
    let throttle_sync_channel = channel.clone();
    actions.insert(
        "throttle-sync",
        Box::new(move |record| {
            ThrottleSyncSink {
                client: ratelimiting::throttle_sync_service_client::ThrottleSyncServiceClient::new(
                    throttle_sync_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
