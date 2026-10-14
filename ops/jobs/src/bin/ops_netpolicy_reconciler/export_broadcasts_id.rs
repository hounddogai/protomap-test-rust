//! `export_broadcasts_id` reads `acme.broadcasts.v1.BroadcastQueryService/SummarizeBroadcasts` and hands what
//! it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::broadcasts::v1 as broadcasts;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source = broadcasts::broadcast_query_service_client::BroadcastQueryServiceClient::new(channel.clone());
    let loaded = source
        .summarize_broadcasts(broadcasts::SummarizeBroadcastsRequest {
            context: Some(context.clone()),
            reason: "primary-export-broadcasts-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let record = BroadcastsIdRecord {
        report_id: loaded.report_id,
        download_uri: loaded.download_uri,
    };
    let mut sinks: BTreeMap<&str, Box<dyn BroadcastsIdWriter>> = BTreeMap::new();
    sinks.insert(
        "certificate-authority-query",
        Box::new(CertificateAuthorityQuerySink {
            client:
                certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient::new(
                    channel.clone(),
                ),
        }),
    );
    sinks.insert(
        "document-validation",
        Box::new(DocumentValidationSink {
            client: documents::document_validation_service_client::DocumentValidationServiceClient::new(
                channel.clone(),
            ),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "key-rotation-lifecycle",
        Box::new(KeyRotationLifecycleSink {
            client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient::new(
                channel,
            ),
        }),
    );
    let Some(sink) = sinks.get("key-rotation-lifecycle") else {
        anyhow::bail!("no sink registered for key-rotation-lifecycle");
    };
    sink.send(record).await
}

/// What `export_broadcasts_id` keeps of each SummarizeBroadcasts response.
#[derive(Clone, Debug, Default)]
pub struct BroadcastsIdRecord {
    pub report_id: String,
    pub download_uri: String,
}

/// Where `export_broadcasts_id` hands each record.
trait BroadcastsIdWriter {
    fn send(&self, record: BroadcastsIdRecord) -> Delivery;
}

/// Hands records to `acme.certificates.v1.CertificateAuthorityQueryService`.
struct CertificateAuthorityQuerySink {
    client: certificates::certificate_authority_query_service_client::CertificateAuthorityQueryServiceClient<Channel>,
}

impl BroadcastsIdWriter for CertificateAuthorityQuerySink {
    fn send(&self, record: BroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .summarize_certificate_authorities(certificates::SummarizeCertificateAuthoritiesRequest {
                    reason: record.report_id,
                    details: Some(certificates::summarize_certificate_authorities_request::Details {
                        format: record.download_uri,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.documents.v1.DocumentValidationService`.
struct DocumentValidationSink {
    client: documents::document_validation_service_client::DocumentValidationServiceClient<Channel>,
    context: common::RequestContext,
}

impl BroadcastsIdWriter for DocumentValidationSink {
    fn send(&self, record: BroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(documents::NormalizeDocumentRequest {
                    document: Some(documents::Document {
                        document_id: record.report_id,
                        ..Default::default()
                    }),
                    reason: record.download_uri,
                    ..Default::default()
                }),
                &context,
            )?;
            client.normalize_document(request).await?;
            Ok(())
        })
    }
}

/// Sends records to `acme.keymanagement.v1.KeyRotationLifecycleService`.
struct KeyRotationLifecycleSink {
    client: keymanagement::key_rotation_lifecycle_service_client::KeyRotationLifecycleServiceClient<Channel>,
}

impl BroadcastsIdWriter for KeyRotationLifecycleSink {
    fn send(&self, record: BroadcastsIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .renew_key_rotation(keymanagement::RenewKeyRotationRequest {
                    reason: record.download_uri,
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
