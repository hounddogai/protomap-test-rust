//! `mirror_document_id` reads `acme.documents.v1.DocumentService/CreateDocument` and hands what it keeps to
//! the sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::common::v1 as common;
use acme_proto::acme::configuration::v1 as configuration;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::patientapp::v1 as patientapp;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = documents::document_service_client::DocumentServiceClient::new(channel.clone());
    let reply = source
        .create_document(documents::CreateDocumentRequest {
            context: Some(context.clone()),
            request_id: "default-mirror-document-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let record = DocumentIdRecord {
        correlation_id: result.correlation_id,
        message: acme_kit::text::normalize(&result.message),
    };
    let sinks: Vec<Box<dyn DocumentIdTarget>> = vec![
        Box::new(ConfigSetValidationSink {
            client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
        Box::new(PatientHomeQuerySink {
            client: patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient::new(channel),
            context: context.clone(),
        }),
    ];
    sinks[0].push(record).await
}

/// What `mirror_document_id` keeps of each CreateDocument response.
#[derive(Clone, Debug, Default)]
pub struct DocumentIdRecord {
    pub correlation_id: String,
    pub message: String,
}

/// Where `mirror_document_id` hands each record.
trait DocumentIdTarget {
    fn push(&self, record: DocumentIdRecord) -> Delivery;
}

/// Hands records to `acme.configuration.v1.ConfigSetValidationService`.
struct ConfigSetValidationSink {
    client: configuration::config_set_validation_service_client::ConfigSetValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl DocumentIdTarget for ConfigSetValidationSink {
    fn push(&self, record: DocumentIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(configuration::ExplainConfigSetDecisionRequest {
                    config_set: Some(configuration::ConfigSet {
                        parent_id: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.explain_config_set_decision(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.patientapp.v1.PatientHomeQueryService`.
struct PatientHomeQuerySink {
    client: patientapp::patient_home_query_service_client::PatientHomeQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl DocumentIdTarget for PatientHomeQuerySink {
    fn push(&self, record: DocumentIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(patientapp::LookupPatientHomeRequest {
                    patient_home_id: record.message,
                    ..Default::default()
                }),
                &context,
            )?;
            client.lookup_patient_home(request).await?;
            Ok(())
        })
    }
}
