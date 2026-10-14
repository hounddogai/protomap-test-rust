//! `copy_flag_evaluation_message` reads
//! `acme.featureflags.v1.FlagEvaluationAdminService/RestoreFlagEvaluation` and hands what it keeps to the
//! sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::fleet::v1 as fleet;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use acme_proto::acme::venues::v1 as venues;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        featureflags::flag_evaluation_admin_service_client::FlagEvaluationAdminServiceClient::new(channel.clone());
    let found = source
        .restore_flag_evaluation(featureflags::RestoreFlagEvaluationRequest {
            context: Some(context.clone()),
            reason: "live-copy-flag-evaluation-message".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let outcome = found.outcome.unwrap_or_default();
    let flag_evaluation = found.flag_evaluation.unwrap_or_default();
    let record = FlagEvaluationMessageRecord {
        message: acme_kit::text::normalize(&result.message),
        status: outcome.status,
        result_status: result.status,
        kind: acme_kit::text::normalize(&flag_evaluation.kind),
    };
    act_on_flag_evaluation_message("role-binding-notification", record, channel, context).await
}

/// What `copy_flag_evaluation_message` keeps of each RestoreFlagEvaluation response.
#[derive(Clone, Debug, Default)]
pub struct FlagEvaluationMessageRecord {
    pub message: String,
    pub status: String,
    pub result_status: String,
    pub kind: String,
}

/// Where `copy_flag_evaluation_message` hands each record.
trait FlagEvaluationMessageSink {
    fn push(&self, record: FlagEvaluationMessageRecord) -> Delivery;
}

/// Writes records to `acme.fleet.v1.VehicleService`.
struct VehicleSink {
    client: fleet::vehicle_service_client::VehicleServiceClient<Channel>,
}

impl FlagEvaluationMessageSink for VehicleSink {
    fn push(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_vehicles(fleet::SearchVehiclesRequest {
                    order_by: record.kind,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitCalculationService`.
struct RateLimitCalculationSink {
    client: ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationMessageSink for RateLimitCalculationSink {
    fn push(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(ratelimiting::ExplainRateLimitCalculationRequest {
                    rate_limit_id: record.status,
                    rate_limit: Some(ratelimiting::RateLimit {
                        description: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_rate_limit_calculation(request).await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.authorization.v1.RoleBindingNotificationService`.
struct RoleBindingNotificationSink {
    client: authorization::role_binding_notification_service_client::RoleBindingNotificationServiceClient<Channel>,
}

impl FlagEvaluationMessageSink for RoleBindingNotificationSink {
    fn push(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .get_role_binding_preferences(authorization::GetRoleBindingPreferencesRequest {
                    role_binding_id: record.result_status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.venues.v1.VenueService`.
struct VenueSink {
    client: venues::venue_service_client::VenueServiceClient<Channel>,
    context: common::RequestContext,
}

impl FlagEvaluationMessageSink for VenueSink {
    fn push(&self, record: FlagEvaluationMessageRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(venues::UpdateVenueRequest {
                    venue: Some(venues::Venue {
                        venue_id: record.result_status,
                        code: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.update_venue(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_flag_evaluation_message(
    kind: &str,
    record: FlagEvaluationMessageRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(FlagEvaluationMessageRecord) -> Delivery>> = BTreeMap::new();
    let vehicle_channel = channel.clone();
    actions.insert(
        "vehicle",
        Box::new(move |record| {
            VehicleSink {
                client: fleet::vehicle_service_client::VehicleServiceClient::new(vehicle_channel.clone()),
            }
            .push(record)
        }),
    );
    let rate_limit_calculation_channel = channel.clone();
    let rate_limit_calculation_context = context.clone();
    actions.insert(
        "rate-limit-calculation",
        Box::new(move |record| {
            RateLimitCalculationSink {
                client: ratelimiting::rate_limit_calculation_service_client::RateLimitCalculationServiceClient::new(
                    rate_limit_calculation_channel.clone(),
                ),
                context: rate_limit_calculation_context.clone(),
            }
            .push(record)
        }),
    );
    let role_binding_notification_channel = channel.clone();
    actions.insert(
        "role-binding-notification",
        Box::new(move |record| {
            RoleBindingNotificationSink {
                client:
                    authorization::role_binding_notification_service_client::RoleBindingNotificationServiceClient::new(
                        role_binding_notification_channel.clone(),
                    ),
            }
            .push(record)
        }),
    );
    let venue_channel = channel.clone();
    let venue_context = context.clone();
    actions.insert(
        "venue",
        Box::new(move |record| {
            VenueSink {
                client: venues::venue_service_client::VenueServiceClient::new(venue_channel.clone()),
                context: venue_context.clone(),
            }
            .push(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
