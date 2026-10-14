//! `relay_get_patient_homes_id` reads `acme.patientapp.v1.PatientHomeService/BatchGetPatientHomes` and hands
//! what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::fulfillment::v1 as fulfillment;
use acme_proto::acme::patientapp::v1 as patientapp;
use acme_proto::acme::ratelimiting::v1 as ratelimiting;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel.clone());
    let fetched = source
        .batch_get_patient_homes(patientapp::BatchGetPatientHomesRequest {
            context: Some(context.clone()),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let record = GetPatientHomesIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
    };
    act_on_get_patient_homes_id("fulfillment-query", record, channel, context).await
}

/// What `relay_get_patient_homes_id` keeps of each BatchGetPatientHomes response.
#[derive(Clone, Debug, Default)]
pub struct GetPatientHomesIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `relay_get_patient_homes_id` hands each record.
trait GetPatientHomesIdWriter {
    fn write(&self, record: GetPatientHomesIdRecord) -> Delivery;
}

/// Sends records to `acme.authorization.v1.AccessPolicyService`.
struct AccessPolicySink {
    client: authorization::access_policy_service_client::AccessPolicyServiceClient<Channel>,
}

impl GetPatientHomesIdWriter for AccessPolicySink {
    fn write(&self, record: GetPatientHomesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .clone_access_policy(authorization::CloneAccessPolicyRequest {
                    access_policy: Some(authorization::AccessPolicy {
                        access_policy_id: record.status,
                        name: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.ratelimiting.v1.RateLimitSyncService`.
struct RateLimitSyncSink {
    client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient<Channel>,
}

impl GetPatientHomesIdWriter for RateLimitSyncSink {
    fn write(&self, record: GetPatientHomesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .reconcile_rate_limits(ratelimiting::ReconcileRateLimitsRequest {
                    details: Some(ratelimiting::reconcile_rate_limits_request::Details {
                        checksum: record.status,
                        peer_region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.fulfillment.v1.FulfillmentQueryService`.
struct FulfillmentQuerySink {
    client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl GetPatientHomesIdWriter for FulfillmentQuerySink {
    fn write(&self, record: GetPatientHomesIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(fulfillment::LookupFulfillmentRequest {
                    fulfillment_id: record.status,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_fulfillment(request).await?;
            Ok(())
        })
    }
}

/// Runs the action registered for kind.
async fn act_on_get_patient_homes_id(
    kind: &str,
    record: GetPatientHomesIdRecord,
    channel: Channel,
    context: common::RequestContext,
) -> anyhow::Result<()> {
    let mut actions: BTreeMap<&str, Box<dyn Fn(GetPatientHomesIdRecord) -> Delivery>> = BTreeMap::new();
    let access_policy_channel = channel.clone();
    actions.insert(
        "access-policy",
        Box::new(move |record| {
            AccessPolicySink {
                client: authorization::access_policy_service_client::AccessPolicyServiceClient::new(
                    access_policy_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let rate_limit_sync_channel = channel.clone();
    actions.insert(
        "rate-limit-sync",
        Box::new(move |record| {
            RateLimitSyncSink {
                client: ratelimiting::rate_limit_sync_service_client::RateLimitSyncServiceClient::new(
                    rate_limit_sync_channel.clone(),
                ),
            }
            .write(record)
        }),
    );
    let fulfillment_query_channel = channel.clone();
    let fulfillment_query_context = context.clone();
    actions.insert(
        "fulfillment-query",
        Box::new(move |record| {
            FulfillmentQuerySink {
                client: fulfillment::fulfillment_query_service_client::FulfillmentQueryServiceClient::new(
                    fulfillment_query_channel.clone(),
                ),
                context: fulfillment_query_context.clone(),
            }
            .write(record)
        }),
    );
    match actions.get(kind) {
        Some(action) => action(record).await,
        None => anyhow::bail!("no action for {kind}"),
    }
}
