//! `backfill_admission_kind` reads `acme.admissions.v1.AdmissionService/CloneAdmission` and hands what it
//! keeps to the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::deliveries::v1 as deliveries;
use acme_proto::acme::itsm::v1 as itsm;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = admissions::admission_service_client::AdmissionServiceClient::new(channel.clone());
    let found = source
        .clone_admission(admissions::CloneAdmissionRequest {
            context: Some(context.clone()),
            request_id: "default-backfill-admission-kind".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let admission = found.admission.unwrap_or_default();
    let record = AdmissionKindRecord {
        kind: admission.kind,
        correlation_id: acme_kit::text::normalize(&admission.correlation_id),
    };
    let sinks: Vec<(&str, Box<dyn AdmissionKindDestination>)> = vec![
        (
            "incident",
            Box::new(IncidentSink {
                client: itsm::incident_service_client::IncidentServiceClient::new(channel.clone()),
            }),
        ),
        (
            "delivery-query",
            Box::new(DeliveryQuerySink {
                client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "config-set-lifecycle",
            Box::new(ConfigSetLifecycleSink {
                client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient::new(
                    channel,
                ),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "delivery-query" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `backfill_admission_kind` keeps of each CloneAdmission response.
#[derive(Clone, Debug, Default)]
pub struct AdmissionKindRecord {
    pub kind: String,
    pub correlation_id: String,
}

/// Where `backfill_admission_kind` hands each record.
trait AdmissionKindDestination {
    fn send(&self, record: AdmissionKindRecord) -> Delivery;
}

/// Writes records to `acme.itsm.v1.IncidentService`.
struct IncidentSink {
    client: itsm::incident_service_client::IncidentServiceClient<Channel>,
}

impl AdmissionKindDestination for IncidentSink {
    fn send(&self, record: AdmissionKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .search_incidents(itsm::SearchIncidentsRequest {
                    filter: Some(itsm::search_incidents_request::Filter {
                        query: record.kind,
                        region: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.deliveries.v1.DeliveryQueryService`.
struct DeliveryQuerySink {
    client: deliveries::delivery_query_service_client::DeliveryQueryServiceClient<Channel>,
}

impl AdmissionKindDestination for DeliveryQuerySink {
    fn send(&self, record: AdmissionKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_deliveries(deliveries::FindDeliveriesRequest {
                    filter: Some(deliveries::find_deliveries_request::Filter {
                        owner_id: record.correlation_id,
                        query: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Hands records to `acme.configuration.v1.ConfigSetLifecycleService`.
struct ConfigSetLifecycleSink {
    client: configuration::config_set_lifecycle_service_client::ConfigSetLifecycleServiceClient<Channel>,
}

impl AdmissionKindDestination for ConfigSetLifecycleSink {
    fn send(&self, record: AdmissionKindRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .expire_config_set(configuration::ExpireConfigSetRequest {
                    details: Some(configuration::expire_config_set_request::Details {
                        reason_code: record.kind,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
