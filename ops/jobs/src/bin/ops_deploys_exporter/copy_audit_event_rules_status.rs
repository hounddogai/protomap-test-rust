//! `copy_audit_event_rules_status` reads `acme.audit.v1.AuditEventValidationService/GetAuditEventRules` and
//! hands what it keeps to the sink of its kind.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use acme_proto::acme::audit::v1 as audit;
use acme_proto::acme::certificates::v1 as certificates;
use acme_proto::acme::common::v1 as common;
use acme_proto::acme::keymanagement::v1 as keymanagement;
use acme_proto::acme::search::v1 as search;
use tonic::transport::Channel;

type Delivery = Pin<Box<dyn Future<Output = anyhow::Result<()>> + Send>>;

pub async fn run(channel: Channel, context: common::RequestContext) -> anyhow::Result<()> {
    let mut source =
        audit::audit_event_validation_service_client::AuditEventValidationServiceClient::new(channel.clone());
    let reply = source
        .get_audit_event_rules(audit::GetAuditEventRulesRequest {
            context: Some(context.clone()),
            audit_event_id: "current-copy-audit-event-rules-status".to_owned(),
            ..Default::default()
        })
        .await?
        .into_inner();
    let result = reply.result.unwrap_or_default();
    let audit_event = reply.audit_event.unwrap_or_default();
    let record = AuditEventRulesStatusRecord {
        status: acme_kit::text::normalize(&result.status),
        audit_event_id: audit_event.audit_event_id,
        correlation_id: result.correlation_id,
        source: audit_event.source,
    };
    let mut sinks: BTreeMap<&str, Box<dyn AuditEventRulesStatusWriter>> = BTreeMap::new();
    sinks.insert(
        "search-index-query",
        Box::new(SearchIndexQuerySink {
            client: search::search_index_query_service_client::SearchIndexQueryServiceClient::new(channel.clone()),
            context: context.clone(),
        }),
    );
    sinks.insert(
        "certificate-query",
        Box::new(CertificateQuerySink {
            client: certificates::certificate_query_service_client::CertificateQueryServiceClient::new(channel.clone()),
        }),
    );
    sinks.insert(
        "key-rotation-workflow",
        Box::new(KeyRotationWorkflowSink {
            client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient::new(channel),
        }),
    );
    let Some(sink) = sinks.get("certificate-query") else {
        anyhow::bail!("no sink registered for certificate-query");
    };
    sink.apply(record).await
}

/// What `copy_audit_event_rules_status` keeps of each GetAuditEventRules response.
#[derive(Clone, Debug, Default)]
pub struct AuditEventRulesStatusRecord {
    pub status: String,
    pub audit_event_id: String,
    pub correlation_id: String,
    pub source: String,
}

/// Where `copy_audit_event_rules_status` hands each record.
trait AuditEventRulesStatusWriter {
    fn apply(&self, record: AuditEventRulesStatusRecord) -> Delivery;
}

/// Writes records to `acme.search.v1.SearchIndexQueryService`.
struct SearchIndexQuerySink {
    client: search::search_index_query_service_client::SearchIndexQueryServiceClient<Channel>,
    context: common::RequestContext,
}

impl AuditEventRulesStatusWriter for SearchIndexQuerySink {
    fn apply(&self, record: AuditEventRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        let context = self.context.clone();
        Box::pin(async move {
            let request = acme_platform::auth::service_request(
                tonic::Request::new(search::SummarizeSearchIndexesRequest {
                    search_index_id: record.status,
                    details: Some(search::summarize_search_indexes_request::Details {
                        cron: record.correlation_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                &context,
            )?;
            client.summarize_search_indexes(request).await?;
            Ok(())
        })
    }
}

/// Writes records to `acme.certificates.v1.CertificateQueryService`.
struct CertificateQuerySink {
    client: certificates::certificate_query_service_client::CertificateQueryServiceClient<Channel>,
}

impl AuditEventRulesStatusWriter for CertificateQuerySink {
    fn apply(&self, record: AuditEventRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .browse_certificates(certificates::BrowseCertificatesRequest {
                    filter: Some(certificates::browse_certificates_request::Filter {
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

/// Sends records to `acme.keymanagement.v1.KeyRotationWorkflowService`.
struct KeyRotationWorkflowSink {
    client: keymanagement::key_rotation_workflow_service_client::KeyRotationWorkflowServiceClient<Channel>,
}

impl AuditEventRulesStatusWriter for KeyRotationWorkflowSink {
    fn apply(&self, record: AuditEventRulesStatusRecord) -> Delivery {
        let mut client = self.client.clone();
        Box::pin(async move {
            client
                .list_key_rotation_workflows(keymanagement::ListKeyRotationWorkflowsRequest {
                    filter: Some(keymanagement::list_key_rotation_workflows_request::Filter {
                        query: record.status,
                        region: record.source,
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .await?;
            Ok(())
        })
    }
}
