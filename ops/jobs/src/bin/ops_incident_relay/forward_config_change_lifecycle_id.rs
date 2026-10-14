//! `forward_config_change_lifecycle_id` reads
//! `acme.configuration.v1.ConfigChangeLifecycleService/GetConfigChangeLifecycle` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::patients::v1 as patients;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_lifecycle_service_client::ConfigChangeLifecycleServiceClient::new(channel.clone());
    let reply = source
        .get_config_change_lifecycle(configuration::GetConfigChangeLifecycleRequest {
            context: Some(context.clone()),
            config_change_id: "live-forward-config-change-lifecycle-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = ConfigChangeLifecycleIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: result.status,
    };
    let sinks: Vec<Box<dyn ConfigChangeLifecycleIdWriter>> = vec![
        Box::new(PatientSink {
            client: patients::patient_service_client::PatientServiceClient::new(channel.clone()),
        }),
        Box::new(ConfigSetCalculationSink {
            client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(
                channel,
            ),
        }),
    ];
    sinks[1].send(record).await
}

/// What `forward_config_change_lifecycle_id` keeps of each GetConfigChangeLifecycle response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeLifecycleIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `forward_config_change_lifecycle_id` hands each record.
trait ConfigChangeLifecycleIdWriter {
    fn send(&self, record: ConfigChangeLifecycleIdRecord) -> Delivery;
}

/// Hands records to `acme.patients.v1.PatientService`.
struct PatientSink {
    client: patients::patient_service_client::PatientServiceClient<Channel>,
}

impl ConfigChangeLifecycleIdWriter for PatientSink {
    fn send(&self, record: ConfigChangeLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_patient(patients::UpdatePatientRequest {
                    patient: Some(patients::Patient {
                        patient_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.configuration.v1.ConfigSetCalculationService`.
struct ConfigSetCalculationSink {
    client: configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient<Channel>,
}

impl ConfigChangeLifecycleIdWriter for ConfigSetCalculationSink {
    fn send(&self, record: ConfigChangeLifecycleIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_config_set_options(configuration::CompareConfigSetOptionsRequest {
                    config_set_id: record.status,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
