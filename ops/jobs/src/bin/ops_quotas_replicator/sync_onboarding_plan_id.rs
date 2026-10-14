//! `sync_onboarding_plan_id` reads
//! `acme.onboarding.v1.OnboardingPlanCalculationService/CalculateOnboardingPlan` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::identity::v1 as identity;
use acme_proto::acme::maintenance::v1 as maintenance;
use acme_proto::acme::onboarding::v1 as onboarding;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        onboarding::onboarding_plan_calculation_service_client::OnboardingPlanCalculationServiceClient::new(
            channel.clone(),
        );
    let found = source
        .calculate_onboarding_plan(onboarding::CalculateOnboardingPlanRequest {
            context: Some(context.clone()),
            reason: "live-sync-onboarding-plan-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let onboarding_plan = found.onboarding_plan.unwrap_or_default();
    let record = OnboardingPlanIdRecord {
        onboarding_plan_id: onboarding_plan.onboarding_plan_id,
        notes: acme_kit::text::normalize(&onboarding_plan.notes),
    };
    deliver_onboarding_plan_id("work-order-calculation", record, channel, &context).await
}

/// What `sync_onboarding_plan_id` keeps of each CalculateOnboardingPlan response.
#[derive(Clone, Debug, Default)]
pub struct OnboardingPlanIdRecord {
    pub onboarding_plan_id: String,
    pub notes: String,
}

/// Where `sync_onboarding_plan_id` hands each record.
trait OnboardingPlanIdDestination {
    fn write(&self, record: OnboardingPlanIdRecord) -> Delivery;
}

/// Sends records to `acme.maintenance.v1.WorkOrderCalculationService`.
struct WorkOrderCalculationSink {
    client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient<Channel>,
}

impl OnboardingPlanIdDestination for WorkOrderCalculationSink {
    fn write(&self, record: OnboardingPlanIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_work_orders(maintenance::RecalculateWorkOrdersRequest {
                    work_order: Some(maintenance::WorkOrder {
                        subject: record.notes,
                        ..Default::default()
                    }),
                    reason: record.onboarding_plan_id,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.careplans.v1.CarePlanQueryService`.
struct CarePlanQuerySink {
    client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient<Channel>,
}

impl OnboardingPlanIdDestination for CarePlanQuerySink {
    fn write(&self, record: OnboardingPlanIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lookup_care_plan(careplans::LookupCarePlanRequest {
                    care_plan_id: record.notes,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.identity.v1.UserQueryService`.
struct UserQuerySink {
    client: identity::user_query_service_client::UserQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl OnboardingPlanIdDestination for UserQuerySink {
    fn write(&self, record: OnboardingPlanIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(identity::SuggestUsersRequest {
                    page: Some(common::PageRequest {
                        page_token: record.onboarding_plan_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.suggest_users(request).await?;
            Ok(())
        })
    }
}

/// Only logs, for kinds no service takes yet.
struct OnboardingPlanIdLogOnly;

impl OnboardingPlanIdDestination for OnboardingPlanIdLogOnly {
    fn write(&self, record: OnboardingPlanIdRecord) -> Delivery {
        Box::pin(async move {
            eprintln!("no sink for {}", record.onboarding_plan_id);
            Ok(())
        })
    }
}

/// Hands a record to the sink of its kind.
async fn deliver_onboarding_plan_id(
    kind: &str,
    record: OnboardingPlanIdRecord,
    channel: Channel,
    context: &common::RequestContext,
) -> anyhow::Result<()> {
    let sink: Box<dyn OnboardingPlanIdDestination> = match kind {
        "work-order-calculation" => Box::new(WorkOrderCalculationSink {
            client: maintenance::work_order_calculation_service_client::WorkOrderCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        "care-plan-query" => Box::new(CarePlanQuerySink {
            client: careplans::care_plan_query_service_client::CarePlanQueryServiceClient::new(channel.clone()),
        }),
        "user-query" => Box::new(UserQuerySink {
            client: identity::user_query_service_client::UserQueryServiceClient::new(channel),
            context: context.clone(),
        }),
        _ => Box::new(OnboardingPlanIdLogOnly),
    };
    sink.write(record).await
}
