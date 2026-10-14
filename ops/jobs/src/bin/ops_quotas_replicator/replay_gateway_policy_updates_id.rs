//! `replay_gateway_policy_updates_id` reads
//! `acme.edgegateway.v1.GatewayPolicyNotificationService/SubscribeGatewayPolicyUpdates` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::edgegateway::v1 as edgegateway;
use acme_proto::acme::patients::v1 as patients;
use acme_proto::acme::personalization::v1 as personalization;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        edgegateway::gateway_policy_notification_service_client::GatewayPolicyNotificationServiceClient::new(
            channel.clone(),
        );
    let fetched = source
        .subscribe_gateway_policy_updates(edgegateway::SubscribeGatewayPolicyUpdatesRequest {
            context: Some(context.clone()),
            reason: "live-replay-gateway-policy-updates-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let record = GatewayPolicyUpdatesIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        reference_id: outcome.reference_id,
        message: result.message,
    };
    act_on_gateway_policy_updates_id("profile-calculation", record, channel, context).await
}

/// What `replay_gateway_policy_updates_id` keeps of each SubscribeGatewayPolicyUpdates response.
#[derive(Clone, Debug, Default)]
pub struct GatewayPolicyUpdatesIdRecord {
    pub correlation_id: String,
    pub reference_id: String,
    pub message: String,
}

/// Where `replay_gateway_policy_updates_id` hands each record.
trait GatewayPolicyUpdatesIdWriter {
    fn send(&self, record: GatewayPolicyUpdatesIdRecord) -> Delivery;
}

/// Writes records to `acme.personalization.v1.ProfileCalculationService`.
struct ProfileCalculationSink {
    client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl GatewayPolicyUpdatesIdWriter for ProfileCalculationSink {
    fn send(&self, record: GatewayPolicyUpdatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(personalization::GetProfileRatesRequest {
                    profile_id: record.reference_id,
                    ..Default::default()
                }),
                &context,
            )?;
            client.get_profile_rates(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.ratelimiting.v1.ThrottleImportService`.
struct ThrottleImportSink {
    client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient<Channel>,
}

impl GatewayPolicyUpdatesIdWriter for ThrottleImportSink {
    fn send(&self, record: GatewayPolicyUpdatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .map_throttle_import_columns(ratelimiting::MapThrottleImportColumnsRequest {
                    reason: record.correlation_id,
                    throttle: Some(ratelimiting::Throttle {
                        throttle_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.patients.v1.PatientCalculationService`.
struct PatientCalculationSink {
    client: patients::patient_calculation_service_client::PatientCalculationServiceClient<Channel>,
    context: common::RequestContext,
}

impl GatewayPolicyUpdatesIdWriter for PatientCalculationSink {
    fn send(&self, record: GatewayPolicyUpdatesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patients::ExplainPatientCalculationRequest {
                    details: Some(patients::explain_patient_calculation_request::Details {
                        locale: record.reference_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_patient_calculation(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_gateway_policy_updates_id(
    kind: &str,
    record: GatewayPolicyUpdatesIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(GatewayPolicyUpdatesIdRecord) -> Delivery>> = BTreeMap::new();
    let profile_calculation_channel = channel.clone();
    let profile_calculation_context = context.clone();
    actions.insert(
        "profile-calculation",
        Box::new(move |record| {
            ProfileCalculationSink {
                client: personalization::profile_calculation_service_client::ProfileCalculationServiceClient::new(
                    profile_calculation_channel.clone(),
                ),
                context: profile_calculation_context.clone(),
            }
            .send(record)
        }),
    );
    let throttle_import_channel = channel.clone();
    actions.insert(
        "throttle-import",
        Box::new(move |record| {
            ThrottleImportSink {
                client: ratelimiting::throttle_import_service_client::ThrottleImportServiceClient::new(
                    throttle_import_channel.clone(),
                ),
            }
            .send(record)
        }),
    );
    let patient_calculation_channel = channel.clone();
    let patient_calculation_context = context.clone();
    actions.insert(
        "patient-calculation",
        Box::new(move |record| {
            PatientCalculationSink {
                client: patients::patient_calculation_service_client::PatientCalculationServiceClient::new(
                    patient_calculation_channel.clone(),
                ),
                context: patient_calculation_context.clone(),
            }
            .send(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
