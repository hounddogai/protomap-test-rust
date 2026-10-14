//! `replay_anomaly_status` reads `acme.anomalies.v1.AnomalyQueryService/LookupAnomaly` and hands what it
//! keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::anomalies::v1 as anomalies;
use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::concessions::v1 as concessions;
use acme_proto::acme::forms::v1 as forms;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = anomalies::anomaly_query_service_client::AnomalyQueryServiceClient::new(channel.clone());
    let loaded = source
        .lookup_anomaly(anomalies::LookupAnomalyRequest {
            context: Some(context.clone()),
            anomaly_id: "live-replay-anomaly-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let anomaly = loaded.anomaly.unwrap_or_default();
    let record = AnomalyStatusRecord {
        status: result.status,
        correlation_id: result.correlation_id,
        subject: acme_kit::text::normalize(&anomaly.subject),
    };
    let mut sinks: BTreeMap<&str, Box<dyn AnomalyStatusSink>> = BTreeMap::new();
    sinks.insert(
        "form-calculation",
        Box::new(FormCalculationSink {
            client: forms::form_calculation_service_client::FormCalculationServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "concession-stand-calculation",
        Box::new(ConcessionStandCalculationSink {
            client:
                concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "role-binding-validation",
        Box::new(RoleBindingValidationSink {
            client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("concession-stand-calculation") else {
        anyhow::bail!("no sink registered for concession-stand-calculation");
    };
    sink.send(record).await
}

/// What `replay_anomaly_status` keeps of each LookupAnomaly response.
#[derive(Clone, Debug, Default)]
pub struct AnomalyStatusRecord {
    pub status: String,
    pub correlation_id: String,
    pub subject: String,
}

/// Where `replay_anomaly_status` hands each record.
trait AnomalyStatusSink {
    fn send(&self, record: AnomalyStatusRecord) -> Delivery;
}

/// Sends records to `acme.forms.v1.FormCalculationService`.
struct FormCalculationSink {
    client: forms::form_calculation_service_client::FormCalculationServiceClient<Channel>,
}

impl AnomalyStatusSink for FormCalculationSink {
    fn send(&self, record: AnomalyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .simulate_form(forms::SimulateFormRequest {
                    form: Some(forms::Form {
                        form_id: record.subject,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.concessions.v1.ConcessionStandCalculationService`.
struct ConcessionStandCalculationSink {
    client: concessions::concession_stand_calculation_service_client::ConcessionStandCalculationServiceClient<Channel>,
}

impl AnomalyStatusSink for ConcessionStandCalculationSink {
    fn send(&self, record: AnomalyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .compare_concession_stand_options(concessions::CompareConcessionStandOptionsRequest {
                    concession_stand: Some(concessions::ConcessionStand {
                        code: record.subject,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.authorization.v1.RoleBindingValidationService`.
struct RoleBindingValidationSink {
    client: authorization::role_binding_validation_service_client::RoleBindingValidationServiceClient<Channel>,
}

impl AnomalyStatusSink for RoleBindingValidationSink {
    fn send(&self, record: AnomalyStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .lint_role_bindings(authorization::LintRoleBindingsRequest {
                    role_binding_id: record.subject,
                    role_binding: Some(authorization::RoleBinding {
                        notes: record.correlation_id,
                        ..Default::default()
                    }),
                    details: Some(authorization::lint_role_bindings_request::Details {
                        locale: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
