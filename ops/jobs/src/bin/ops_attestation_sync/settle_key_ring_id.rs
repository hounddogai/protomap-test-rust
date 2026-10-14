//! `settle_key_ring_id` reads `acme.keymanagement.v1.KeyRingApprovalService/SubmitKeyRing` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        keymanagement::key_ring_approval_service_client::KeyRingApprovalServiceClient::new(channel.clone());
    let fetched = source
        .submit_key_ring(keymanagement::SubmitKeyRingRequest {
            context: Some(context.clone()),
            key_ring_id: "primary-settle-key-ring-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let key_ring = fetched.key_ring.unwrap_or_default();
    let result = fetched.result.unwrap_or_default();
    let record = KeyRingIdRecord {
        parent_id: key_ring.parent_id,
        message: result.message,
    };
    let mut sinks: BTreeMap<String, Box<dyn KeyRingIdSink>> = BTreeMap::new();
    sinks.insert(
        "config-change-calculation".to_owned(),
        Box::new(ConfigChangeCalculationSink {
            client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient::new(
                channel.clone(),
            ),
        }),
    );
    sinks.insert(
        "patient-query".to_owned(),
        Box::new(PatientQuerySink {
            client: patients::patient_query_service_client::PatientQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    );
    let target = std::env::var("SETTLE_KEY_RING_ID_TARGET").unwrap_or_default();
    match sinks.get(&target) {
        Some(sink) => sink.forward(record).await,
        None => Ok(()),
    }
}

/// What `settle_key_ring_id` keeps of each SubmitKeyRing response.
#[derive(Clone, Debug, Default)]
pub struct KeyRingIdRecord {
    pub parent_id: String,
    pub message: String,
}

/// Where `settle_key_ring_id` hands each record.
trait KeyRingIdSink {
    fn forward(&self, record: KeyRingIdRecord) -> Delivery;
}

/// Sends records to `acme.configuration.v1.ConfigChangeCalculationService`.
struct ConfigChangeCalculationSink {
    client: configuration::config_change_calculation_service_client::ConfigChangeCalculationServiceClient<Channel>,
}

impl KeyRingIdSink for ConfigChangeCalculationSink {
    fn forward(&self, record: KeyRingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .estimate_config_change(configuration::EstimateConfigChangeRequest {
                    config_change: Some(configuration::ConfigChange {
                        source: record.parent_id,
                        config_change_id: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.patients.v1.PatientQueryService`.
struct PatientQuerySink {
    client: patients::patient_query_service_client::PatientQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl KeyRingIdSink for PatientQuerySink {
    fn forward(&self, record: KeyRingIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patients::FindPatientsRequest {
                    page: Some(common::PageRequest {
                        page_token: record.parent_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.find_patients(request).await?;
            Ok(())
        })
    }
}
