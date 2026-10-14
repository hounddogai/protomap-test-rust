//! `relay_config_change_id` reads `acme.configuration.v1.ConfigChangeApprovalService/ApproveConfigChange` and
//! hands what it keeps to `acme.patientapp.v1.PatientHomeService`.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_change_approval_service_client::ConfigChangeApprovalServiceClient::new(channel.clone());
    let reply = source
        .approve_config_change(configuration::ApproveConfigChangeRequest {
            context: Some(context.clone()),
            reason: "current-relay-config-change-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let config_change = reply.config_change.unwrap_or_default();
    let result = reply.result.unwrap_or_default();
    let record = ConfigChangeIdRecord {
        correlation_id: config_change.correlation_id,
        kind: acme_kit::text::normalize(&config_change.kind),
        message: acme_kit::text::normalize(&result.message),
    };
    PatientHomeSink {
        client: patientapp::patient_home_service_client::PatientHomeServiceClient::new(channel),
    }
    .send(record)
    .await
}

/// What `relay_config_change_id` keeps of each ApproveConfigChange response.
#[derive(Clone, Debug, Default)]
pub struct ConfigChangeIdRecord {
    pub correlation_id: String,
    pub kind: String,
    pub message: String,
}

/// Where `relay_config_change_id` hands each record.
trait ConfigChangeIdSink {
    fn send(&self, record: ConfigChangeIdRecord) -> Delivery;
}

/// Sends records to `acme.patientapp.v1.PatientHomeService`.
struct PatientHomeSink {
    client: patientapp::patient_home_service_client::PatientHomeServiceClient<Channel>,
}

impl ConfigChangeIdSink for PatientHomeSink {
    fn send(&self, record: ConfigChangeIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .update_patient_home(patientapp::UpdatePatientHomeRequest {
                    patient_home: Some(patientapp::PatientHome {
                        patient_home_id: record.kind,
                        subject: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
