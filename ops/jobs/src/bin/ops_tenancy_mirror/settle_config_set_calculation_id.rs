//! `settle_config_set_calculation_id` reads
//! `acme.configuration.v1.ConfigSetCalculationService/ExplainConfigSetCalculation` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::admissions::v1 as admissions;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::documents::v1 as documents;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        configuration::config_set_calculation_service_client::ConfigSetCalculationServiceClient::new(channel.clone());
    let loaded = source
        .explain_config_set_calculation(configuration::ExplainConfigSetCalculationRequest {
            context: Some(context.clone()),
            reason: "primary-settle-config-set-calculation-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = loaded.result.unwrap_or_default();
    let outcome = loaded.outcome.unwrap_or_default();
    let record = ConfigSetCalculationIdRecord {
        correlation_id: acme_kit::text::normalize(&result.correlation_id),
        status: acme_kit::text::normalize(&outcome.status),
    };
    let sinks: Vec<(&str, Box<dyn ConfigSetCalculationIdSink>)> = vec![
        (
            "document-query",
            Box::new(DocumentQuerySink {
                client: documents::document_query_service_client::DocumentQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "admission",
            Box::new(AdmissionSink {
                client: admissions::admission_service_client::AdmissionServiceClient::new(channel),
                context: context.clone(),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "document-query" {
            continue;
        }
        sink.push(record.clone()).await?;
    }
    Ok(())
}

/// What `settle_config_set_calculation_id` keeps of each ExplainConfigSetCalculation response.
#[derive(Clone, Debug, Default)]
pub struct ConfigSetCalculationIdRecord {
    pub correlation_id: String,
    pub status: String,
}

/// Where `settle_config_set_calculation_id` hands each record.
trait ConfigSetCalculationIdSink {
    fn push(&self, record: ConfigSetCalculationIdRecord) -> Delivery;
}

/// Hands records to `acme.documents.v1.DocumentQueryService`.
struct DocumentQuerySink {
    client: documents::document_query_service_client::DocumentQueryServiceClient<Channel>,
}

impl ConfigSetCalculationIdSink for DocumentQuerySink {
    fn push(&self, record: ConfigSetCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_documents(documents::BrowseDocumentsRequest {
                    filter: Some(documents::browse_documents_request::Filter {
                        owner_id: record.status,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Passes records to `acme.admissions.v1.AdmissionService`.
struct AdmissionSink {
    client: admissions::admission_service_client::AdmissionServiceClient<Channel>,
    context: common::RequestContext,
}

impl ConfigSetCalculationIdSink for AdmissionSink {
    fn push(&self, record: ConfigSetCalculationIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(admissions::SearchAdmissionsRequest {
                    filter: Some(admissions::search_admissions_request::Filter {
                        owner_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.search_admissions(request).await?;
            Ok(())
        })
    }
}
