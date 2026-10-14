//! `carry_role_binding_export_id` reads
//! `acme.authorization.v1.RoleBindingExportService/ShareRoleBindingExport` and hands what it keeps to the
//! sink of its kind.

use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::authorization::v1 as authorization;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::documents::v1 as documents;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        authorization::role_binding_export_service_client::RoleBindingExportServiceClient::new(channel.clone());
    let fetched = source
        .share_role_binding_export(authorization::ShareRoleBindingExportRequest {
            context: Some(context.clone()),
            role_binding_id: "current-carry-role-binding-export-id".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = fetched.result.unwrap_or_default();
    let outcome = fetched.outcome.unwrap_or_default();
    let role_binding = fetched.role_binding.unwrap_or_default();
    let record = RoleBindingExportIdRecord {
        correlation_id: result.correlation_id,
        message: outcome.message,
        notes: acme_kit::text::normalize(&role_binding.notes),
    };
    let sinks: Vec<(&str, Box<dyn RoleBindingExportIdDestination>)> = vec![
        (
            "document-query",
            Box::new(DocumentQuerySink {
                client: documents::document_query_service_client::DocumentQueryServiceClient::new(channel.clone()),
            }),
        ),
        (
            "crypto-key-approval",
            Box::new(CryptoKeyApprovalSink {
                client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient::new(channel),
            }),
        ),
    ];
    for (kind, sink) in &sinks {
        if *kind != "document-query" {
            continue;
        }
        sink.send(record.clone()).await?;
    }
    Ok(())
}

/// What `carry_role_binding_export_id` keeps of each ShareRoleBindingExport response.
#[derive(Clone, Debug, Default)]
pub struct RoleBindingExportIdRecord {
    pub correlation_id: String,
    pub message: String,
    pub notes: String,
}

/// Where `carry_role_binding_export_id` hands each record.
trait RoleBindingExportIdDestination {
    fn send(&self, record: RoleBindingExportIdRecord) -> Delivery;
}

/// Passes records to `acme.documents.v1.DocumentQueryService`.
struct DocumentQuerySink {
    client: documents::document_query_service_client::DocumentQueryServiceClient<Channel>,
}

impl RoleBindingExportIdDestination for DocumentQuerySink {
    fn send(&self, record: RoleBindingExportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .find_documents(documents::FindDocumentsRequest {
                    filter: Some(documents::find_documents_request::Filter {
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

/// Writes records to `acme.keymanagement.v1.CryptoKeyApprovalService`.
struct CryptoKeyApprovalSink {
    client: keymanagement::crypto_key_approval_service_client::CryptoKeyApprovalServiceClient<Channel>,
}

impl RoleBindingExportIdDestination for CryptoKeyApprovalSink {
    fn send(&self, record: RoleBindingExportIdRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .escalate_crypto_key(keymanagement::EscalateCryptoKeyRequest {
                    crypto_key: Some(keymanagement::CryptoKey {
                        currency_code: record.correlation_id,
                        ..Default::default()
                    }),
                    reason: record.notes,
                    details: Some(keymanagement::escalate_crypto_key_request::Details {
                        approver_email: record.message,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
