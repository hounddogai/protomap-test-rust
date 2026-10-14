//! `settle_care_plans_id` reads `acme.careplans.v1.CarePlanQueryService/ResolveCarePlans` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::careplans::v1 as careplans;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::personalization::v1 as personalization;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = careplans::care_plan_query_service_client::CarePlanQueryServiceClient::new(channel.clone());
    let fetched = source
        .resolve_care_plans(careplans::ResolveCarePlansRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let page_info = fetched.page_info.unwrap_or_default();
    let record = CarePlansIdRecord {
        correlation_id: result.correlation_id,
        message: result.message,
        status: result.status,
        next_page_token: page_info.next_page_token,
    };
    let mut sinks: BTreeMap<&str, Box<dyn CarePlansIdTarget>> = BTreeMap::new();
    sinks.insert(
        "config-change-workflow",
        Box::new(ConfigChangeWorkflowSink {
            client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "profile-calculation",
        Box::new(ProfileCalculationSink {
            client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient::new(channel),
        }),
    );
    match sinks.get("profile-calculation") {
        Some(sink) => sink.write(record).await,
        None => anyhow::bail!("no sink registered for profile-calculation"),
    }
}

/// What `settle_care_plans_id` keeps of each ResolveCarePlans response.
#[derive(Clone, Debug, Default)]
pub struct CarePlansIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub status: String,
    pub next_page_token: String,
}

/// Where `settle_care_plans_id` hands each record.
trait CarePlansIdTarget {
    fn write(&self, record: CarePlansIdRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigChangeWorkflowService`.
struct ConfigChangeWorkflowSink {
    client: configuration::config_change_workflow_service_client::ConfigChangeWorkflowServiceClient<Channel>,
    context: common::RequestContext,
}

impl CarePlansIdTarget for ConfigChangeWorkflowSink {
    fn write(&self, record: CarePlansIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::InspectConfigChangeWorkflowRequest {
                    config_change_id: record.next_page_token,
                    ..Default::default()
                }),
                &context,
            )?;
            client.inspect_config_change_workflow(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.personalization.v1.ProfileCalculationService`.
struct ProfileCalculationSink {
    client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient<Channel>,
}

impl CarePlansIdTarget for ProfileCalculationSink {
    fn write(&self, record: CarePlansIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_profile(personalization::EstimateProfileRequest {
                    details: Some(personalization::estimate_profile_request::Details {
                        currency_code: record.next_page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
