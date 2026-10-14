//! `replay_admissions_id` reads `acme.admissions.v1.AdmissionCalculationService/RecalculateAdmissions` and
//! hands what it keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::featureflags::v1 as featureflags;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::workplaces::v1 as workplaces;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        admissions::admission_calculation_service_client::AdmissionCalculationServiceClient::new(channel.clone());
    let found = source
        .recalculate_admissions(admissions::RecalculateAdmissionsRequest {
            context: Some(context.clone()),
            reason: "primary-replay-admissions-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = found.result.unwrap_or_default();
    let admission = found.admission.unwrap_or_default();
    let record = AdmissionsIdRecord {
        correlation_id: result.correlation_id,
        status: result.status,
        kind: admission.kind,
        source: admission.source,
    };
    let sinks: Vec<Box<dyn AdmissionsIdSink>> = vec![
        Box::new(KeyRingValidationSink {
            client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(FlagRolloutCalculationSink {
            client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
        Box::new(BuildingQuerySink {
            client: workplaces::building_query_service_client::BuildingQueryServiceClient::new(channel),
        }),
    ];
    sinks[1].push(record).await
}

/// What `replay_admissions_id` keeps of each RecalculateAdmissions response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionsIdRecord {
    pub correlation_id: String,
    pub status: String,
    pub kind: String,
    pub source: String,
}

/// Where `replay_admissions_id` hands each record.
trait AdmissionsIdSink {
    fn push(&self, record: AdmissionsIdRecord) -> Delivery;
}

/// Sends records to `acme.keymanagement.v1.KeyRingValidationService`.
struct KeyRingValidationSink {
    client: keymanagement::key_ring_validation_service_client::KeyRingValidationServiceClient<Channel>,
}

impl AdmissionsIdSink for KeyRingValidationSink {
    fn push(&self, record: AdmissionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .check_key_ring_eligibility(keymanagement::CheckKeyRingEligibilityRequest {
                    key_ring_id: record.source,
                    key_ring: Some(keymanagement::KeyRing {
                        key_ring_id: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.featureflags.v1.FlagRolloutCalculationService`.
struct FlagRolloutCalculationSink {
    client: featureflags::flag_rollout_calculation_service_client::FlagRolloutCalculationServiceClient<Channel>,
}

impl AdmissionsIdSink for FlagRolloutCalculationSink {
    fn push(&self, record: AdmissionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .recalculate_flag_rollouts(featureflags::RecalculateFlagRolloutsRequest {
                    flag_rollout_id: record.source,
                    flag_rollout: Some(featureflags::FlagRollout {
                        flag_rollout_id: record.kind,
                        title: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.workplaces.v1.BuildingQueryService`.
struct BuildingQuerySink {
    client: workplaces::building_query_service_client::BuildingQueryServiceClient<Channel>,
}

impl AdmissionsIdSink for BuildingQuerySink {
    fn push(&self, record: AdmissionsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .suggest_buildings(workplaces::SuggestBuildingsRequest {
                    filter: Some(workplaces::suggest_buildings_request::Filter {
                        query: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
